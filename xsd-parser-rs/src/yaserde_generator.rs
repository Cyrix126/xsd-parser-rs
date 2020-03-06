use crate::generator::Generator;
use crate::parser::types::{
    Enum, EnumCase, RsFile, Struct, StructField, StructFieldSource, TupleStruct,
};
use roxmltree::Namespace;
use std::borrow::Cow;

pub struct YaserdeGenerator<'input> {
    target_ns: Option<Namespace<'input>>,
}

impl<'input> YaserdeGenerator<'input> {
    pub fn new(schema: &RsFile<'input>) -> Self {
        YaserdeGenerator {
            target_ns: schema.target_ns.clone(),
        }
    }
}

impl<'input> Generator for YaserdeGenerator<'input> {
    fn target_ns(&self) -> &Option<Namespace<'_>> {
        &self.target_ns
    }

    fn tuple_struct_macro(&self, _: &TupleStruct) -> Cow<'static, str> {
        "#[derive(Default, PartialEq, Debug, UtilsTupleSerDe)]\n".into()
    }

    fn struct_macro(&self, _: &Struct) -> Cow<'static, str> {
        let derives = "#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]\n";
        match &self.target_ns {
            Some(tn) => match tn.name() {
                Some(name) => format!(
                    "{derives}#[yaserde(prefix = \"{prefix}\", namespace = \"{prefix}: {uri}\")]\n",
                    derives = derives,
                    prefix = name,
                    uri = tn.uri()
                ),
                None => format!(
                    "{derives}#[yaserde(namespace = \"{uri}\")]\n",
                    derives = derives,
                    uri = tn.uri()
                ),
            },
            None => format!("{derives}#[yaserde()]\n", derives = derives),
        }
        .into()
    }

    fn enum_macro(&self, _: &Enum) -> Cow<'static, str> {
        "#[derive(PartialEq, Debug, YaSerialize, YaDeserialize)]".into()
    }

    fn struct_field_macro(&self, sf: &StructField) -> Cow<'static, str> {
        match sf.source {
            StructFieldSource::Choice => yaserde_for_flatten_element().into(),
            StructFieldSource::Attribute => yaserde_for_attribute(sf.name.as_str()).into(),
            StructFieldSource::Element => {
                yaserde_for_element(sf.name.as_str(), self.target_ns.as_ref()).into()
            }
            _ => "".into(),
        }
    }

    fn enum_case_macro(&self, ec: &EnumCase) -> Cow<'static, str> {
        yaserde_rename_macro(ec.name.as_str()).into()
    }
}

fn yaserde_for_attribute(name: &str) -> String {
    if let Some(index) = name.find(':') {
        format!(
            "    #[yaserde(attribute, prefix = \"{}\" rename = \"{}\")]\n",
            &name[0..index],
            &name[index + 1..]
        )
    } else {
        format!("    #[yaserde(attribute, rename = \"{}\")]\n", name)
    }
}

fn yaserde_for_element(name: &str, target_namespace: Option<&roxmltree::Namespace>) -> String {
    let (prefix, field_name) = if let Some(index) = name.find(':') {
        (Some(&name[0..index]), &name[index + 1..])
    } else {
        (target_namespace.and_then(|ns| ns.name()), name)
    };

    match prefix {
        Some(p) => format!(
            "    #[yaserde(prefix = \"{}\", rename = \"{}\")]\n",
            p, field_name
        ),
        None => format!("    #[yaserde(rename = \"{}\")]\n", field_name),
    }
}

fn yaserde_rename_macro(name: &str) -> String {
    format!("    #[yaserde(rename = \"{}\")]\n", name)
}

fn yaserde_for_flatten_element() -> String {
    "    #[yaserde(flatten)]\n".to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_yaserde_for_element() {
        let doc = roxmltree::Document::parse("<e xmlns:tt='http://www.w3.org'/>").unwrap();

        let ns = Some(&doc.root_element().namespaces()[0]);
        assert_eq!(
            yaserde_for_element("xop:Include", ns),
            "    #[yaserde(prefix = \"xop\", rename = \"Include\")]\n"
        );
    }

    #[test]
    fn test_yaserde_rename_macro() {
        assert_eq!(
            yaserde_rename_macro("NTP"),
            "    #[yaserde(rename = \"NTP\")]\n"
        )
    }
}