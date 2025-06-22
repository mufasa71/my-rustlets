use git2::{DiffFormat, DiffLineType, DiffOptions, Repository};

pub fn get_diff() -> Result<String, git2::Error> {
    let repo = Repository::open_from_env()?;

    let head = repo.head()?;
    let head_oid = head
        .target()
        .ok_or_else(|| git2::Error::from_str("Head has no target"))?;
    let commit = repo.find_commit(head_oid)?;
    let tree = commit.tree()?;
    let mut diff_options = DiffOptions::new();
    diff_options.ignore_whitespace(true);
    diff_options.ignore_whitespace_change(true);
    diff_options.ignore_whitespace_eol(true);

    let diff = repo.diff_tree_to_index(Some(&tree), None, Some(&mut diff_options))?;
    let mut buf = Vec::new();

    diff.print(DiffFormat::Patch, |_, _, line| {
        let origin = line.origin_value();
        match origin {
            DiffLineType::Addition => buf.extend_from_slice(b"+"),
            DiffLineType::Deletion => buf.extend_from_slice(b"-"),
            DiffLineType::HunkHeader => return true,
            _ => {}
        }

        buf.extend(line.content());
        true
    })?;

    let contents = String::from_utf8_lossy(&buf);

    Ok(contents.to_string())
}
