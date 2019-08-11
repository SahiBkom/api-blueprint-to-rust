use log::*;
use pulldown_cmark::{Event, Options, Parser, Tag};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::prelude::*;

mod field;

#[derive(Eq, PartialEq, Debug)]
struct CodeStruct {
    fields: BTreeMap<String, field::Field>,
}

impl CodeStruct {
    fn new() -> CodeStruct {
        CodeStruct {
            fields: BTreeMap::new(),
        }
    }

    fn generate(&self, name: &String) -> codegen::Struct {
        // let help = String::new()
        // write!(help, "{} - {}");
        let mut s = codegen::Struct::new(&name);
        s.derive("Debug");
        s.derive("Serialize");
        s.derive("Deserialize");
        for (name, field) in &self.fields {
            s.field(name, &field.field_type);
        }
        s
    }
}

#[derive(Eq, PartialEq, Debug)]
struct Code {
    structs: BTreeMap<String, CodeStruct>,
}

impl Code {
    fn new() -> Code {
        Code {
            structs: BTreeMap::new(),
        }
    }

    fn add_struct_field(&mut self, struct_name: String, field: &field::Field) {
        let struct_name = struct_name.replace(" ", "").replace("-", "_");
        trace!("name: {:?}", field);
        let s = self.structs.entry(struct_name).or_insert(CodeStruct::new());
        s.fields.insert(field.name.clone(), field.clone());
    }

    fn generate(&self) -> String {
        let mut cg = codegen::Scope::new();
        cg.import("serde", "{Deserialize, Serialize}");
        for (name, code_struct) in &self.structs {
            cg.push_struct(code_struct.generate(&name));
        }

        cg.to_string()
    }
}

fn main() {
    env_logger::init();
    info!("starting up");

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        print!("Wrong number of args, need only the apib file");
        return;
    }

    let mut file = File::open(&args[1]).unwrap();
    let mut markdown_input = String::new();
    file.read_to_string(&mut markdown_input).unwrap();

    #[derive(Debug, Clone)]
    enum Action {
        Header(String),
        Fields(String),
        FieldAdd(String, field::FieldBuilder),
        SkipList(String, u32),
        // FieldType(String, String),
        None,
    }

    let mut code = Code::new();

    let mut action: Action = Action::None;
    for event in Parser::new_ext(&markdown_input,  Options::all()) {
        match (event, action.clone()) {
            (Event::Start(Tag::Header(_)), _) => {
                action = Action::Header(String::new());
            }
            (Event::End(Tag::Header(_)), Action::Header(_)) => action = Action::None,
            // (Event::End(Tag::Header(_)), _) => action = Action::None,
            (Event::Code(code), Action::Header(_)) => {
                action = Action::Header(code.to_string());
            }
            (Event::Text(text), Action::Header(header)) => {
                if text.ends_with(" (object)") {
                    // Attendee
                    debug!("== {}{} ==", header, text);
                    action =
                        Action::Fields(format!("{}{}", header, text.trim_end_matches(" (object)")));
                }
            }
            (Event::Start(Tag::Item), Action::Fields(t)) => {
                action = Action::FieldAdd(t, field::FieldBuilder::new())
            }
            (Event::Code(code), Action::FieldAdd(class, field)) => {
                let mut field = field.clone();
                field.add_code(code.to_string());
                action = Action::FieldAdd(class, field);
            }
            (Event::Text(code), Action::FieldAdd(class, field)) => {
                let mut field = field.clone();
                field.add_text(code.to_string());
                action = Action::FieldAdd(class, field);
            }
            (Event::Start(Tag::List(_)), Action::FieldAdd(class, field)) => {
                // Don't support enum's or sub types
                action =  Action::SkipList(class.clone(), 1);
                debug!("- {:?}", field);
                if let Some(field) = field.build() {
                    code.add_struct_field(class.clone(), &mut field.clone());
                }
            }
            (Event::Start(Tag::List(_)), Action::SkipList(class, cnt)) => {
                action =  Action::SkipList(class.clone(), cnt + 1)
            }
            (Event::End(Tag::List(_)), Action::SkipList(class, cnt)) => {
                if cnt == 1 {
                    action = Action::Fields(class.clone());
                } else {
                    action = Action::SkipList(class.clone(), cnt - 1);
                }
            }
            (Event::End(Tag::Item), Action::FieldAdd(class, field)) => {
                action = Action::Fields(class.clone());
                debug!("- {:?}", field);
                if let Some(field) = field.build() {
                    code.add_struct_field(class.clone(), &mut field.clone());
                }
            }
            (a, Action::Fields(_)) => debug!("{:?}", a),
            _ => (),
        }
    }

    // debug!("code {:?}", code);
    println!("{}", code.generate());
}
