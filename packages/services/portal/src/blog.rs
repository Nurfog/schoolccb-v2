use chrono::NaiveDate;
use std::fs;

#[derive(Clone, Debug)]
pub struct BlogPost {
    pub slug: String,
    pub title: String,
    pub date: NaiveDate,
    pub excerpt: String,
    pub author: String,
    pub html: String,
}

pub fn load_blog_posts() -> Vec<BlogPost> {
    let mut posts = Vec::new();
    let blog_dir = format!("{}/blog", env!("CARGO_MANIFEST_DIR"));

    let entries = match fs::read_dir(&blog_dir) {
        Ok(e) => e,
        Err(_) => {
            tracing::warn!("Blog directory not found: {blog_dir}");
            return posts;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to read blog post {:?}: {e}", path);
                continue;
            }
        };

        match parse_post(&content) {
            Ok(post) => posts.push(post),
            Err(e) => tracing::error!("Failed to parse blog post {:?}: {e}", path),
        }
    }

    posts.sort_by(|a, b| b.date.cmp(&a.date));
    posts
}

fn parse_post(content: &str) -> Result<BlogPost, String> {
    let content = content.trim();
    if !content.starts_with("---") {
        return Err("Missing frontmatter".into());
    }

    let end = content[3..]
        .find("\n---")
        .ok_or_else(|| "Unclosed frontmatter".to_string())?;
    let frontmatter = &content[3..3 + end];
    let body = &content[3 + end + 4..];

    let mut title = String::new();
    let mut date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let mut excerpt = String::new();
    let mut author = String::from("SchoolCBB");

    for line in frontmatter.lines() {
        if let Some(val) = line.strip_prefix("title: ").map(|v| v.trim().trim_matches('"')) {
            title = val.to_string();
        } else if let Some(val) = line.strip_prefix("date: ") {
            date = NaiveDate::parse_from_str(val.trim(), "%Y-%m-%d")
                .map_err(|e| format!("Invalid date {val}: {e}"))?;
        } else if let Some(val) = line.strip_prefix("excerpt: ").map(|v| v.trim().trim_matches('"')) {
            excerpt = val.to_string();
        } else if let Some(val) = line.strip_prefix("author: ").map(|v| v.trim().trim_matches('"')) {
            author = val.to_string();
        }
    }

    if title.is_empty() {
        return Err("Missing title in frontmatter".into());
    }

    let slug = slugify(&title);
    let parser = pulldown_cmark::Parser::new(body);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    let html = format!("<div class=\"blog-content\">{html}</div>");

    Ok(BlogPost {
        slug,
        title,
        date,
        excerpt,
        author,
        html,
    })
}

fn slugify(title: &str) -> String {
    title
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != ' ', "")
        .replace(' ', "-")
        .trim_matches('-')
        .to_string()
}
