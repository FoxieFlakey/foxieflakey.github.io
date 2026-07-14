pub fn sanify_path(path: &str) -> String {
    let mut segments = Vec::new();

    for segment in path.split('/') {
        if segment == "." {
            continue;
        } else if segment == ".." {
            segments.pop();
        } else if !segment.is_empty() {
            segments.push(segment);
        }
    }

    let joined = segments.join("/");
    format!(
        "{}{}{}",
        if joined.starts_with('/') { "" } else { "/" },
        joined,
        if path.ends_with('/') && joined.len() > 1 { "/" } else { "" }
    )
}
