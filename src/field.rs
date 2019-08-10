#[derive(Eq, PartialEq, Debug, Clone)]
pub struct FieldBuilder {
    todo: String,
    name: Option<String>,
    example: Option<String>,
    field_type: Option<String>,
    doc: Option<String>,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Field {
    pub name: String,
    pub example: String,
    pub field_type: String,
    pub doc: String,
}

impl FieldBuilder {
    pub fn new() -> FieldBuilder {
        FieldBuilder {
            todo: String::new(),
            name: None,
            example: None,
            field_type: None,
            doc: None,
        }
    }

    pub fn add_code(&mut self, code: String) {
        self.todo = format!("{}{}", self.todo, code);
        // println!("C {:?} {:?} {:?} {:?}", code, self.todo, self.name, self. example);
    }

    pub fn add_text(&mut self, text: String) {
        let mut text = text.clone();
        if self.name.is_none() {
            if let Some(pos) = text.find(":") {
                let add: String = text.drain(..pos).collect();
                self.name = Some(format!("{}{}", self.todo, add));
                text.drain(..1); // remove :
                self.todo = String::new();
            }
            // println!("A {:?} {:?} {:?} {:?}", text, self.todo, self.name, self. example);
        }
        if self.name.is_none() || self.example.is_none() {
            if let Some(pos) = text.find("(") {
                let add: String = text.drain(..pos).collect();
                if self.name.is_none() {
                    self.name = Some(format!("{}{}", self.todo, add));
                    self.example = Some("".to_string());
                } else {
                    self.example = Some(format!("{}{}", self.todo, add));
                }
                text.drain(..1); // remove (
                self.todo = String::new();
            }
            // println!("B {:?} {:?} {:?} {:?}", text, self.todo, self.name, self. example);
        }
        if self.example.is_some() && self.field_type.is_none() {
            if let Some(pos) = text.find(")") {
                let add: String = text.drain(..pos).collect();
                self.field_type = Some(format!("{}{}", self.todo, add));
                self.todo = String::new();
            }
            // println!("B {:?} {:?} {:?} {:?}", text, self.todo, self.name, self. example);
        }
        if self.field_type.is_some() {
            if let Some(ref mut doc) = self.doc {
                *doc = format!("{}{}{}", doc, self.todo, text);
                self.todo = String::new();
            } else {
                if let Some(pos) = text.find("-") {
                    text.drain(..(pos + 1));
                    self.doc = Some(text);
                }
            }
        } else {
            self.todo = format!("{}{}", self.todo, text);
        }
    }

    pub fn build(&self) -> Option<Field> {
        if let (Some(name), Some(example), Some(field_type), Some(doc)) = (
            self.name.clone(),
            self.example.clone(),
            self.field_type.clone(),
            self.doc.clone(),
        ) {
            Some(Field {
                name,
                example,
                field_type,
                doc,
            })
        } else {
            log::error!("wrong field {:?}", self);
            None
        }
    }
}
// id: 12345 (string) - Attendee ID
// created: `2018-05-12T02:00:00Z` (datetime) - When the attendee was created (order placed)
// + team (Attendee Team, nullable) - The attendee’s team information
fn main_test() {
    let mut t = FieldBuilder::new();
    t.add_text(String::from("id: 12345 (string) - Attendee ID"));
    println!("{:?}", t);
    let mut t = FieldBuilder::new();
    t.add_text(String::from("created: "));
    t.add_code(String::from("2018-05-12T02:00:00Z"));
    t.add_text(String::from(
        "(datetime) - When the attendee was created (order placed)",
    ));
    println!("{:?}", t);
    let mut t = FieldBuilder::new();
    t.add_text(String::from(
        "team (Attendee Team, nullable) - The attendee’s team information",
    ));
    println!("{:?}", t);
}
