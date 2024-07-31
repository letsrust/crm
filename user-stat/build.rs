use anyhow::Result;
// use proto_builder_trait::tonic::BuilderAttributes;
use std::fs;

fn main() -> Result<()> {
    fs::create_dir_all("src/pb")?;

    let builder: tonic_build::Builder = tonic_build::configure();
    builder
        .build_client(true)
        .build_server(true)
        .out_dir("src/pb")
        .with_sqlx_from_row(&["User"], None)
        .compile(
            &[
                "../protos/user-stats/messages.proto",
                "../protos/user-stats/rpc.proto",
            ],
            &["../protos/"],
        )
        .unwrap();

    Ok(())
}

pub trait BuilderAttrs {
    fn with_sqlx_from_row(self, paths: &[&str], extra_attrs: Option<&[&str]>) -> Self;
    fn with_type_attributes(self, paths: &[&str], attributes: &[&str]) -> Self;
    fn with_optional_type_attributes(self, paths: &[&str], attributes: Option<&[&str]>) -> Self;
}

impl BuilderAttrs for tonic_build::Builder {
    fn with_sqlx_from_row(self, paths: &[&str], extra_attrs: Option<&[&str]>) -> Self {
        paths.iter().fold(self, |builder, ty| {
            builder
                .type_attribute(ty, sqlx_from_row_attr())
                .with_optional_type_attributes(&[ty], extra_attrs)
        })
    }

    fn with_type_attributes(self, paths: &[&str], attributes: &[&str]) -> Self {
        let attr = attributes.join("\n");

        paths.iter().fold(self, |builder, ty| {
            builder.type_attribute(ty, attr.as_str())
        })
    }

    fn with_optional_type_attributes(self, paths: &[&str], attributes: Option<&[&str]>) -> Self {
        if let Some(attributes) = attributes {
            self.with_type_attributes(paths, attributes)
        } else {
            self
        }
    }
}

pub fn serde_attr(ser: bool, de: bool) -> &'static str {
    match (ser, de) {
        (true, true) => "#[derive(serde::Serialize, serde::Deserialize)]",
        (true, false) => "#[derive(serde::Serialize)]",
        (false, true) => "#[derive(serde::Deserialize)]",
        (false, false) => "",
    }
}

pub fn serde_as_attr() -> &'static str {
    "#[serde_with::serde_as]\n#[serde_with::skip_serializing_none]"
}

pub fn sqlx_type_attr() -> &'static str {
    "#[derive(sqlx::Type)]"
}

pub fn sqlx_from_row_attr() -> &'static str {
    "#[derive(sqlx::FromRow)]"
}

pub fn derive_builder_attr() -> &'static str {
    "#[derive(derive_builder::Builder)]\n#[builder(setter(into, strip_option), default)]"
}
