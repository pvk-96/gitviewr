use crate::git::errors::{AnalysisError, Result};
use crate::models::analysis::AnalysisData;

pub fn write(data: &AnalysisData, path: &str) -> Result<()> {
    let html = build_html(data);
    std::fs::write(path, html)
        .map_err(|e| AnalysisError::ExportFailed(format!("Could not write the HTML file: {e}")))?;
    Ok(())
}

fn build_html(data: &AnalysisData) -> String {
    let mut html = String::with_capacity(1024 * 1024);
    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    html.push_str(&format!(
        "<title>GitViewr Report – {}</title>\n",
        e(&data.repository.name)
    ));
    html.push_str("<style>\n");
    html.push_str(include_str!("report.css"));
    html.push_str("\n</style>\n");
    html.push_str("</head>\n<body>\n");
    html.push_str("<header>\n<h1>GitViewr Report</h1>\n");
    html.push_str(&format!(
        "<p class=\"subtitle\">{}</p>\n",
        e(&data.repository.name)
    ));
    html.push_str(&format!(
        "<p class=\"meta\">Generated {}</p>\n",
        e(&crate::utils::dates::now_iso())
    ));
    html.push_str("</header>\n\n");
    html.push_str("<section>\n<h2>Repository</h2>\n<table>\n");
    row(&mut html, "Name", &data.repository.name);
    row(&mut html, "Path", &data.metadata.path);
    row(
        &mut html,
        "Source Type",
        &format!("{:?}", data.repository.source_type),
    );
    if let Some(branch) = &data.stats.branch {
        row(&mut html, "Current Branch", branch);
    }
    row(&mut html, "Commits", &data.stats.commit_count.to_string());
    row(&mut html, "Branches", &data.stats.branch_count.to_string());
    row(
        &mut html,
        "Tracked Files",
        &data.stats.tracked_files.to_string(),
    );
    row(
        &mut html,
        "Contributors",
        &data.stats.contributor_count.to_string(),
    );
    html.push_str("</table>\n</section>\n\n");
    html.push_str("<section>\n<h2>Changes Summary</h2>\n<table>\n");
    row(
        &mut html,
        "Total Additions",
        &format!("+{}", data.stats.total_additions),
    );
    row(
        &mut html,
        "Total Deletions",
        &format!("-{}", data.stats.total_deletions),
    );
    row(&mut html, "Net Change", &data.stats.net_change.to_string());
    html.push_str("</table>\n</section>\n\n");
    if !data.analytics.contributors.is_empty() {
        html.push_str("<section>\n<h2>Contributors</h2>\n<table>\n");
        html.push_str(
            "<tr><th>Name</th><th>Commits</th><th>Additions</th><th>Deletions</th></tr>\n",
        );
        for c in &data.analytics.contributors {
            html.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>+{}</td><td>-{}</td></tr>\n",
                e(&c.name),
                c.commits,
                c.additions,
                c.deletions
            ));
        }
        html.push_str("</table>\n</section>\n\n");
    }

    if !data.analytics.commits_over_time.is_empty() {
        html.push_str("<section>\n<h2>Commits Over Time</h2>\n<table>\n");
        html.push_str(
            "<tr><th>Period</th><th>Commits</th><th>Additions</th><th>Deletions</th></tr>\n",
        );
        for b in &data.analytics.commits_over_time {
            html.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>+{}</td><td>-{}</td></tr>\n",
                e(&b.period),
                b.commits,
                b.additions,
                b.deletions
            ));
        }
        html.push_str("</table>\n</section>\n\n");
    }

    if !data.commits.is_empty() {
        html.push_str("<section>\n<h2>Commits</h2>\n<table>\n");
        html.push_str("<tr><th>Hash</th><th>Date</th><th>Author</th><th>Message</th><th>+</th><th>-</th></tr>\n");
        for c in &data.commits {
            let hash = if c.hash.len() > 7 {
                &c.hash[..7]
            } else {
                &c.hash
            };
            html.push_str(&format!(
                "<tr><td class=\"hash\">{}</td><td>{}</td><td>{}</td><td>{}</td><td>+{}</td><td>-{}</td></tr>\n",
                e(hash), e(&c.author_date), e(&c.author_name), e(&c.subject), c.additions, c.deletions
            ));
        }
        html.push_str("</table>\n</section>\n\n");
    }

    html.push_str("</body>\n</html>");
    html
}

fn row(html: &mut String, label: &str, value: &str) {
    html.push_str(&format!(
        "<tr><td class=\"label\">{label}</td><td>{}</td></tr>\n",
        e(value)
    ));
}

/// HTML-escape a string.
fn e(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 20);
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}
