use std::fs::File;
use std::io::prelude::*;
use pulldown_cmark::{Event, Options, Parser, Tag};
use std::collections::HashMap;
use log::*;
use env_logger::Logger;

#[derive(Hash, Eq, PartialEq, Debug)]
struct CodeField {
    pub field_type: String, 
    pub doc: String,
}

impl CodeField {
    fn new() -> CodeField {
        CodeField {
            field_type: String::new(),
            doc: String::new(),
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
struct CodeStruct {
    fields: HashMap<String, CodeField>,
}

impl CodeStruct {
    fn new() -> CodeStruct {
        CodeStruct {
            fields: HashMap::new(),
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
            s.field(&name, &field.field_type);
        }
        s
    }
}

#[derive(Eq, PartialEq, Debug)]
struct Code {
    structs: HashMap<String, CodeStruct>
}

impl Code {
    fn new() -> Code {
        Code {
            structs: HashMap::new(),
        }
    }

    fn add_struct_field(&mut self, struct_name: String, field_name: String, field_type:String) {
        let struct_name = struct_name.replace(" ", "");
        trace!("name: {}", field_name);
        let s = self.structs.entry(struct_name).or_insert(CodeStruct::new());
        s.fields.entry(field_name).or_insert(CodeField::new()).field_type = field_type;
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
        Header,
        Fields(String),
        FieldName(String),
        FieldType(String, String),
        None,
    }

    let mut code = Code::new();

    let mut o = Options::all();
    let mut action: Action = Action::None;
    for event in Parser::new_ext(&markdown_input, o) {
        match (event, action.clone()) {
            (Event::Start(Tag::Header(x)), _) => {
                action = Action::Header;
            }
            // (Event::End(Tag::Header(_)), _) => action = Action::None,
            (Event::Text(text), Action::Header) => {
                if text.ends_with(" Fields") {
                    debug!("== {} ==", text);
                    action = Action::Fields(text.trim_end_matches(" Fields").to_string());
                }
            }
            // (Event::Text(text), Action::Fields(_)) => println!("Text {}", text),
            // (Event::Start(Tag::Table(_)), _) => println!("Table"),
            // (Event::Start(Tag::TableHead), _) => println!("Head"),
            // (Event::End(Tag::TableHead), _) => println!("Head <"),
            (Event::Start(Tag::TableRow), Action::Fields(class)) => action = Action::FieldName(class),
            // (Event::Start(Tag::TableCell),_) => println!(" - TableCell"),
            (Event::Code(field_name), Action::FieldName(class)) => {
    //            println!(" - {} code:{}", name, code);
                action = Action::FieldType(class, field_name.to_string());
            }
            (Event::Code(field_type), Action::FieldType(class, field_name)) => {
                debug!(" - {} {} {}", class, field_name, field_type);
                code.add_struct_field(class.clone(), field_name, field_type.to_string());
                action = Action::Fields(class);
            }
            _ => (),
        }
    }

    println!("{}", code.generate());
}
