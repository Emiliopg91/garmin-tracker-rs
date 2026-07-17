use std::{
    collections::{BTreeMap, HashMap, HashSet},
    env,
    fs::{self},
    path::PathBuf,
};

use swc_common::{FileName, SourceMap, sync::Lrc};
use swc_ecma_ast::{Callee, EsVersion, Expr, Lit, Module};
use swc_ecma_parser::{Lexer, Parser, StringInput, Syntax, TsSyntax};
use swc_ecma_visit::{Visit, VisitWith};
use walkdir::WalkDir;

fn main() {
    tauri_build::build();
    generate_ddl_file();
    let translations_file = generate_translations_file();
    generate_translations_typescript(translations_file);
}

fn generate_ddl_file() {
    let ddls_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .join("resources")
        .join("ddl");
    println!("cargo:rerun-if-changed={}", ddls_dir.display());

    let files = fs::read_dir(ddls_dir)
        .unwrap()
        .filter_map(|e| {
            if let Ok(val) = e {
                Some(val.path())
            } else {
                None
            }
        })
        .collect::<Vec<PathBuf>>();

    let mut output = "#[derive(Clone)]\npub struct DdlVersion {\n    pub version: u16,\n    pub description: &'static str,\n    pub sql: &'static str\n}".to_string();
    output.push_str(&format!(
        "pub static DDLS: [DdlVersion; {}] = [",
        files.len()
    ));
    for path in files {
        let name = path.file_name().unwrap().display().to_string();
        let version: u16 = name.split("_").next().unwrap().parse().unwrap();
        let content = fs::read_to_string(&path).unwrap();
        let description = match content.find('\n') {
            Some(pos) => {
                let comment = content[..pos].to_string();
                comment.strip_prefix("--").unwrap().trim().to_string()
            }
            None => unreachable!(),
        };

        output.push_str(&format!(
            "DdlVersion{{version: {}, description:\"{}\", sql:include_str!(\"{}\")}},\n",
            version,
            description,
            path.display()
        ));
    }
    output.push_str("];");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = PathBuf::from(&out_dir).join("database_versions.rs");
    fs::write(&dest_path, output).unwrap();
}

fn generate_translations_file() -> HashMap<String, String> {
    let translations_file = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .join("resources")
        .join("translations.yaml");
    println!("cargo:rerun-if-changed={}", translations_file.display());

    let content = fs::read_to_string(translations_file).unwrap();
    let translations_map: BTreeMap<String, BTreeMap<String, String>> =
        serde_yaml::from_str(content.as_str()).unwrap();

    let default_lang = "en";

    let mut lang_var = std::env::var("LANG").unwrap_or("C".to_string());
    lang_var = lang_var.to_lowercase();
    if lang_var == "c" {
        lang_var = default_lang.to_string();
    }
    if lang_var.contains(".") {
        lang_var = lang_var.split(".").next().unwrap().into();
    }
    if lang_var.contains("_") {
        lang_var = lang_var.split("_").next().unwrap().into();
    }

    let mut translations = HashMap::new();
    for (key, values) in &translations_map {
        translations.insert(
            key.clone(),
            values
                .get(&lang_var)
                .unwrap_or(values.get(default_lang).unwrap_or(&key.to_string()))
                .clone(),
        );
    }

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = PathBuf::from(&out_dir).join("translations.yaml");
    fs::write(&dest_path, serde_yaml::to_string(&translations).unwrap()).unwrap();

    println!("cargo:rustc-env=TRANSLATIONS_YAML={}", dest_path.display());

    translations
}

fn generate_translations_typescript(translations: HashMap<String, String>) {
    struct TsTranslationVisitor {
        pub translation_keys: HashSet<String>,
    }

    impl Visit for TsTranslationVisitor {
        fn visit_call_expr(&mut self, node: &swc_ecma_ast::CallExpr) {
            if let Callee::Expr(expr) = &node.callee
                && let Expr::Ident(id) = &**expr
                && id.sym == "translate"
                && let Some(first_arg) = node.args.first()
                && let Expr::Lit(Lit::Str(str_lit)) = &*first_arg.expr
            {
                self.translation_keys
                    .insert(str_lit.value.to_string_lossy().to_string());
            }
        }
    }

    let mut visitor = TsTranslationVisitor {
        translation_keys: HashSet::new(),
    };

    let front_src_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .join("src");
    for entry in WalkDir::new(&front_src_dir)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if path.is_file()
            && matches!(
                path.extension().and_then(|s| s.to_str()),
                Some("ts" | "tsx")
            )
        {
            println!("cargo:rerun-if-changed={}", path.display());
            let source = std::fs::read_to_string(path).unwrap();
            let module = parse_module(&source);
            module.visit_with(&mut visitor);
        }
    }

    fn parse_module(source: &str) -> Module {
        let cm: Lrc<SourceMap> = Default::default();
        let fm = cm.new_source_file(
            FileName::Custom("file.ts".into()).into(),
            source.to_string(),
        );
        let lexer = Lexer::new(
            Syntax::Typescript(TsSyntax {
                tsx: true,
                decorators: true,
                ..Default::default()
            }),
            EsVersion::latest(),
            StringInput::from(&*fm),
            None,
        );
        let mut parser = Parser::new_from(lexer);
        parser.parse_module().unwrap()
    }

    let mut keys = visitor
        .translation_keys
        .into_iter()
        .collect::<Vec<String>>();
    keys.sort();

    let mut content = "export const TRANSLATIONS: Record<string, string> = {\n".to_string();
    for key in &keys {
        if let Some(translation) = translations.get(key) {
            content.push_str(&format!("\t\"{}\": \"{}\",\n", key, translation));
        }
    }
    content.push_str("};");

    fs::write(front_src_dir.join("utils").join("translations.ts"), content).unwrap();
}
