//! The `odopen://sync/?...` link SharePoint's Sync button opens, cut into what a library needs.

/// One document library to sync.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncLink {
    pub site_id: String,
    pub list_id: String,
    /// The last segment of `webUrl`, the site's short name.
    pub site: String,
    pub list_title: String,
}

impl SyncLink {
    /// The folder and profile name, `<Site> - <Library>`, as the Windows client names it.
    #[must_use]
    pub fn name(&self) -> String {
        format!("{} - {}", self.site, self.list_title)
    }
}

/// None unless every needed parameter is there and the scheme is `odopen://`.
#[must_use]
pub fn parse(url: &str) -> Option<SyncLink> {
    let query = url.strip_prefix("odopen://sync/?")?;
    let param = |key: &str| {
        query
            .split('&')
            .find_map(|pair| pair.strip_prefix(key)?.strip_prefix('='))
            .map(decode)
    };
    let web_url = param("webUrl")?;
    Some(SyncLink {
        site_id: param("siteId")?.trim_matches(['{', '}']).to_owned(),
        list_id: param("listId")?.trim_matches(['{', '}']).to_owned(),
        site: web_url.trim_end_matches('/').rsplit('/').next()?.to_owned(),
        list_title: param("listTitle")?,
    })
}

/// Percent-decoding, with `+` as a space. A bad escape is kept as typed.
fn decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while let Some(&b) = bytes.get(i) {
        let escaped = (b == b'%')
            .then(|| bytes.get(i + 1..i + 3))
            .flatten()
            .and_then(|hex| u8::from_str_radix(std::str::from_utf8(hex).ok()?, 16).ok());
        match escaped {
            Some(byte) => {
                out.push(byte);
                i += 3;
            }
            None => {
                out.push(if b == b'+' { b' ' } else { b });
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_links_name_the_library() {
        let link = parse(
            "odopen://sync/?siteId=%7Bd3a1%7D&webId=%7Bw%7D&listId=%7Bl-1%7D\
             &userEmail=a%40b.c&webUrl=https%3A%2F%2Ft.sharepoint.com%2Fsites%2FOps&listTitle=Shared+Documents",
        )
        .unwrap();
        assert_eq!(link.site_id, "d3a1");
        assert_eq!(link.list_id, "l-1");
        assert_eq!(link.name(), "Ops - Shared Documents");
        assert_eq!(parse("odopen://sync/?siteId=x"), None);
        assert_eq!(parse("https://example.com"), None);
    }
}
