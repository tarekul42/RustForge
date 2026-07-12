use rand::Rng;

/// Convert a human-readable title into a URL-safe slug.
///
/// Replaces non-alphanumeric characters (except hyphens) with spaces,
/// trims and collapses whitespace into single hyphens, and limits
/// the output to 128 characters.
pub fn generate_slug(title: &str) -> String {
    let slug: String = title
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == ' ' {
                c
            } else {
                ' '
            }
        })
        .collect();

    let slug: Vec<&str> = slug.split_whitespace().collect();
    let mut result = slug.join("-");
    if result.is_empty() {
        result = "untitled".to_string();
    }
    if result.len() > 128 {
        result.truncate(128);
        let trimmed_end = result.trim_end_matches('-').to_string();
        if trimmed_end.is_empty() {
            result
        } else {
            trimmed_end
        }
    } else {
        result
    }
}

/// Ensure a slug is unique by appending a suffix if needed.
/// Calls `check(slug)` which should return `true` if the slug is available.
/// Retries up to `max_attempts` times with random suffixes.
pub async fn ensure_unique_slug<F, Fut>(
    base_slug: &str,
    check: F,
    max_attempts: u32,
) -> Result<String, String>
where
    F: Fn(String) -> Fut,
    Fut: std::future::Future<Output = Result<bool, String>>,
{
    if check(base_slug.to_string()).await.unwrap_or(false) {
        return Ok(base_slug.to_string());
    }

    for _ in 1..max_attempts {
        let suffix: u32 = rand::rng().random_range(1000..9999);
        let candidate = format!("{base_slug}-{suffix}");
        if check(candidate.clone()).await.unwrap_or(false) {
            return Ok(candidate);
        }
    }

    Err(format!(
        "could not generate unique slug after {max_attempts} attempts"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_slug() {
        assert_eq!(generate_slug("Hello World"), "hello-world");
    }

    #[test]
    fn slug_with_special_chars() {
        assert_eq!(generate_slug("Rust & WebDev 101!"), "rust-webdev-101");
    }

    #[test]
    fn empty_slug_fallback() {
        assert_eq!(generate_slug("!!!"), "untitled");
    }

    #[test]
    fn slug_truncation() {
        let long = "a".repeat(200);
        let slug = generate_slug(&long);
        assert!(slug.len() <= 128);
    }

    #[tokio::test]
    async fn ensure_unique_first_try() {
        let expected = "hello".to_string();
        let result = ensure_unique_slug(
            "hello",
            move |s| {
                let expected = expected.clone();
                async move { Ok(s == expected) }
            },
            3,
        )
        .await;
        assert_eq!(result.unwrap(), "hello");
    }

    #[tokio::test]
    async fn ensure_unique_retry() {
        let result = ensure_unique_slug("hello", |s| async move { Ok(s != "hello") }, 3).await;
        assert!(result.is_ok());
        assert!(result.unwrap().starts_with("hello-"));
    }
}
