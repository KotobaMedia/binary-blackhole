use km_to_sql::metadata::ColumnMetadata;

pub fn format_column(column: &ColumnMetadata) -> String {
    let mut out = format!("  - `{}`", column.name);
    if let Some(ref desc) = column.desc {
        out.push_str(&format!(": {}", desc));
    }
    let mut annotations = vec![format!("type: {}", column.data_type)];
    if let Some(fk) = &column.foreign_key {
        annotations.push(format!(
            r#"foreign key: "{}"."{}""#,
            fk.foreign_table, fk.foreign_column
        ));
    }
    if let Some(enum_vs) = &column.enum_values {
        let mut enum_v_strs = vec![];
        for enum_v in enum_vs {
            let mut str = format!("`{}`", enum_v.value);
            if let Some(desc) = &enum_v.desc {
                str.push_str(&format!(": {}", desc));
            }
            enum_v_strs.push(str);
        }
        let enum_v_str = enum_v_strs.join(", ");
        annotations.push(format!("possible values: {}", enum_v_str));
    }
    out.push_str(&format!(" ({})", annotations.join(", ")));
    out.push('\n');
    out
}
