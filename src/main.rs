//! CLI 工具：接收一个 .ts 文件路径，解析并将所有普通字符串字面量（不包括模板字符串的 quasis）替换为按顺序递增的索引字符串 "0","1",...
//! 输出两个文件：`<name>_r.ts`（替换后的 TS）与 `<name>_s.json`（映射表，形如 {"0":"原始字符串0","1":"原始字符串1",...}）
//!
//! 使用说明：
//!   sb_dice <path/to/file.ts>
//!
//! 错误处理：
//!   - 如果参数不对或不是以 `.ts` 结尾，会打印错误并返回非零退出码。
//!   - 解析或写文件失败会打印错误并返回非零退出码。
//!
//! 备注：不会替换模板字符串中的静态部分（quasis）；会替换 import/require 中的字符串模块路径。
//!      输出代码中去掉注释（通过 emitter.comments = None 控制）。

use std::env;
use std::fs;
use std::path::Path;
use std::process;

use swc_core::common::{sync::Lrc, FileName, SourceMap};
use swc_core::ecma::ast::Str;
use swc_core::ecma::codegen::{text_writer::JsWriter, Emitter, Config};
use swc_core::ecma::parser::{lexer::Lexer, Parser, StringInput, Syntax};
use swc_core::ecma::visit::{VisitMut, VisitMutWith};
use swc_core::ecma::ast::EsVersion;

use serde_json::Map;
use serde_json::Value;

/// 替换器：记录计数并收集原始字符串（按顺序）
struct StringReplacer {
    counter: usize,
    originals: Vec<String>,
}

impl StringReplacer {
    fn new() -> Self {
        Self {
            counter: 0,
            originals: Vec::new(),
        }
    }
}

impl VisitMut for StringReplacer {
    fn visit_mut_str(&mut self, n: &mut Str) {
        // 只针对 Str 节点（这不会匹配模板的 quasis，模板静态文本是 TplElement）
        // 使用字符串的原始值，而不是 Debug 格式（避免生成带转义的双引号）
        let original = n.value.as_str().unwrap_or_default().to_string();
        // 记录原始内容
        self.originals.push(original);

        // 生成新的字符串值，例如 "0", "1", ...
        let new_val = self.counter.to_string();
        n.value = new_val.into();

        // 清除 raw，强制 codegen 使用新的 value
        n.raw = None;

        self.counter += 1;
    }
}

fn print_help() {
    println!(r#"sb_dice - 用来解决 DICE 的 sb 字符串机制的 字符串提取与替换工具

Author: shenjack & Gemini 3 Pro & GPT 5 mini & GLM 4.7 & DeepSeek v3.2 (按照贡献多少排序(确信))

用法:
  sb_dice <path/to/file.ts>
  sb_dice -h
  sb_dice --help

选项:
  -h, --help    显示此帮助信息

参数:
  <path/to/file.ts>  输入的 TypeScript 文件路径

说明:
  解析 TypeScript 文件，将所有普通字符串字面量（不包括模板字符串的 quasis）
  替换为按顺序递增的索引字符串 "0","1",...

输出:
  生成两个文件：
    - <name>_r.ts  : 替换后的 TS 文件
    - <name>_s.json: 映射表，形如 {{"0":"原始字符串0","1":"原始字符串1",...}}

注意事项:
  - 不会替换模板字符串中的静态部分（quasis） (反正你也用不到)
  - 会替换 import/require 中的字符串模块路径 (反正也不应该有)
  - 输出代码中去掉注释"#);
}

fn print_usage_and_exit() -> ! {
    eprintln!(r#"错误：缺少参数

使用 'sb_dice -h' 或 'sb_dice --help' 查看详细帮助信息"#);
    process::exit(1);
}

fn main() {
    // 解析命令行参数
    let mut args = env::args().skip(1);

    // 检查帮助参数
    let arg = args.next();
    if arg.as_deref() == Some("-h") || arg.as_deref() == Some("--help") {
        print_help();
        process::exit(0);
    }

    let input_path = match arg {
        Some(p) => p,
        None => {
            print_usage_and_exit();
        }
    };

    // 确保是 .ts 文件
    let path = Path::new(&input_path);
    if path.extension().and_then(|s| s.to_str()) != Some("ts") {
        eprintln!("错误：仅支持 .ts 文件作为输入：{}", input_path);
        eprintln!("使用 'sb_dice -h' 或 'sb_dice --help' 查看帮助信息");
        process::exit(2);
    }

    // 读取文件内容
    let src = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("读取文件失败 {}: {}", input_path, e);
            eprintln!("使用 'sb_dice -h' 或 'sb_dice --help' 查看帮助信息");
            process::exit(3);
        }
    };

    // --- 解析 ---
    let cm: Lrc<SourceMap> = Default::default();
    // 使用真实文件名，方便解析错误定位
    let fm = cm.new_source_file(FileName::Real(path.to_path_buf()).into(), src);

    let lexer = Lexer::new(
        Syntax::Typescript(Default::default()),
        EsVersion::Es2020,
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);

    let mut module = match parser.parse_module() {
        Ok(m) => m,
        Err(err) => {
            eprintln!("解析 TypeScript 文件失败: {:?}", err);
            eprintln!("使用 'sb_dice -h' 或 'sb_dice --help' 查看帮助信息");
            process::exit(4);
        }
    };

    // --- 遍历并替换 ---
    let mut replacer = StringReplacer::new();
    module.visit_mut_with(&mut replacer);

    // --- 代码生成（去掉注释） ---
    let mut buf = vec![];

    {
        let writer = JsWriter::new(cm.clone(), "\n", &mut buf, None);

        let mut emitter = Emitter {
            cfg: Config::default(),
            cm: cm.clone(),
            comments: None, // 去掉注释
            wr: writer,
        };

        if let Err(e) = emitter.emit_module(&module) {
            eprintln!("生成代码失败: {:?}", e);
            process::exit(5);
        }
    }

    let output_code = match String::from_utf8(buf) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("输出编码转换失败: {}", e);
            process::exit(6);
        }
    };

    // --- 写入输出文件 ---
    // 构造输出文件名：原名_r.ts 与 原名_s.json
    let stem = match path.file_stem().and_then(|s| s.to_str()) {
        Some(s) => s.to_string(),
        None => {
            eprintln!("无法解析输入文件名");
            process::exit(7);
        }
    };

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let out_ts_path = parent.join(format!("{}_r.ts", stem));
    let out_json_path = parent.join(format!("{}_s.json", stem));

    // 写 ts 文件
    if let Err(e) = fs::write(&out_ts_path, output_code) {
        eprintln!("写入输出 TS 文件失败 {}: {}", out_ts_path.display(), e);
        process::exit(8);
    }

    // 生成 JSON 映射：{"0": "原始0", "1": "原始1", ...}
    let mut map = Map::new();
    for (idx, orig) in replacer.originals.iter().enumerate() {
        map.insert(idx.to_string(), Value::String(orig.clone()));
    }

    let json_text = match serde_json::to_string_pretty(&Value::Object(map)) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("生成 JSON 失败: {}", e);
            process::exit(9);
        }
    };

    if let Err(e) = fs::write(&out_json_path, json_text) {
        eprintln!("写入输出 JSON 文件失败 {}: {}", out_json_path.display(), e);
        process::exit(10);
    }

    println!(
        "成功：生成 {} 与 {}",
        out_ts_path.display(),
        out_json_path.display()
    );
}
