use log::*;

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
        trace!(
            "add_code {:?} {:?} {:?} {:?}",
            code,
            self.todo,
            self.name,
            self.example
        );
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
            trace!(
                "add_text A {:?} {:?} {:?} {:?}",
                text,
                self.todo,
                self.name,
                self.example
            );
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
            trace!(
                "add_text B {:?} {:?} {:?} {:?}",
                text,
                self.todo,
                self.name,
                self.example
            );
        }
        if self.example.is_some() && self.field_type.is_none() {
            if let Some(pos) = text.find(")") {
                let add: String = text.drain(..pos).collect();
                self.field_type = Some(format!("{}{}", self.todo, add));
                self.todo = String::new();
            }
            trace!(
                "add_text C {:?} {:?} {:?} {:?}",
                text,
                self.todo,
                self.name,
                self.example
            );
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
        if let (Some(name), Some(example), Some(field_type)) = (
            self.name.clone(),
            self.example.clone(),
            self.field_type.clone(),
        ) {
            std::dbg!(Some(Field {
                name: fix_field_name(name),
                example,
                field_type: fix_field_type(field_type),
                doc: self.doc.clone().unwrap_or_default(),
            }))
        } else {
            log::error!("wrong field {:?}", self);
            None
        }
    }
}

fn fix_field_name(field_name: String) -> String {
    match field_name.trim().as_ref() {
        "type" => "r#type".to_string(),
        name => name.replace("-", "_"),
    }
}

fn fix_field_type(field_type: String) -> String {
    let mut is_vec = false;
    let mut is_optional = false;
    let mut is_nullable = false;
    let mut is_enum = false;
    // let mut is_base_type = false;
    let mut is_required = false;

    let mut split = field_type.split(",");

    let mut value: String = if let Some(ref mut a) = split.next() {
        is_vec = a.starts_with("array[");
        if is_vec {
            *a = a.trim_start_matches("array[");
            *a = a.trim_end_matches("]");
        }
        is_enum = a.starts_with("enum[");
        if is_enum {
            *a = a.trim_start_matches("enum[");
            *a = a.trim_end_matches("]");
        }
        let a = &a.replace(" ", "").replace("-", "_");

        if a.is_empty() {
            "String".to_string() // TODO, spec? + `subquestion_ids` (array[])
        } else {
            match a.as_ref() {
                "string" => "String".to_string(),
                "enum" => "String".to_string(),   // TODO
                "object" => "String".to_string(), // TODO
                "array" => "String".to_string(),  // TODO
                "boolean" => "bool".to_string(),
                "number" => "i64".to_string(),
                "datetime" => "String".to_string(), // TODO
                a => a.to_string(),
            }
        }
    } else {
        String::new()
    };

    while let Some(e) = split.next() {
        match e.trim() {
            "optional" => is_optional = true,
            "nullable" => is_nullable = true,
            "required" => is_required = true,
            _ => (),
        }
    }

    if is_vec {
        value = format!("Vec<{}>", value);
    }

    if is_optional || is_nullable {
        value = format!("Option<{}>", value);
    }

    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_field_name() {
        assert_eq!("r#type".to_string(), fix_field_name("type".to_string()));
        assert_eq!(
            "datetime_tz_utc,".to_string(),
            fix_field_name("datetime-tz-utc,".to_string())
        );
    }

    // TODO make a real test of it
    #[test]
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
        let mut t = FieldBuilder::new();
        t.add_text(String::from(
            "event (Event, optional) - Full details of the event (requires the ",
        ));
        t.add_code(String::from("event"));
        t.add_text(String::from("expansion"));
        println!("{:?}", t);

        let mut t = FieldBuilder::new();
        t.add_text(String::from("questions (array"));
        t.add_text(String::from("["));
        t.add_text(String::from("Question"));
        t.add_text(String::from("]"));
        t.add_text(String::from(
            ", optional) - The per-attendee custom questions",
        ));
        println!("{:?}", t);
    }

    // TODO make a real test of it
    #[test]
    fn test_fix_field_type() {
        println!(
            "{}",
            fix_field_type(String::from("Attendee Assigned Unit, optional, nullable"))
        );
        println!(
            "{}",
            fix_field_type(String::from("array[Answer], optional"))
        );
        println!(
            "{}",
            fix_field_type(String::from("array[enum[string]], optional"))
        );
    }
}
