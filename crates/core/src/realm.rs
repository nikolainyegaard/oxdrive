//! Microsoft's home realm discovery answer for a sign-in address. The sign-in page asks the same
//! unauthenticated endpoint before it knows where to send the browser; the reply names the tenant,
//! which the client never prints. See docs/client.md.

/// The flyout title for a realm reply: `OneDrive - <tenant>` for work and school accounts,
/// `OneDrive - Personal` for Microsoft accounts, `None` when the reply names no brand.
#[must_use]
pub fn title(json: &str) -> Option<String> {
    let brand = field(json, "FederationBrandName")?;
    if brand == "Windows Live" {
        return Some("OneDrive - Personal".into());
    }
    Some(format!("OneDrive - {brand}"))
}

/// The string value of a top-level key. Enough JSON for this one reply: escapes are honoured,
/// nesting is not.
fn field(json: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\":\"");
    let start = json.find(&needle)? + needle.len();
    let mut chars = json.get(start..)?.chars();
    let mut out = String::new();
    loop {
        match chars.next()? {
            '"' => return Some(out),
            '\\' => match chars.next()? {
                'u' => {
                    let hex: String = chars.by_ref().take(4).collect();
                    let code = u32::from_str_radix(&hex, 16).ok()?;
                    out.push(char::from_u32(code)?);
                }
                'n' => out.push('\n'),
                't' => out.push('\t'),
                other => out.push(other),
            },
            c => out.push(c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORK: &str = r#"{"State":4,"UserState":1,"Login":"user@example.com","NameSpaceType":"Managed","DomainName":"example.com","FederationBrandName":"Example AS","CloudInstanceName":"microsoftonline.com","CloudInstanceIssuerUri":"urn:federation:MicrosoftOnline"}"#;
    const PERSONAL: &str = r#"{"State":3,"UserState":2,"Login":"user@outlook.com","NameSpaceType":"Federated","DomainName":"live.com","FederationGlobalVersion":-1,"AuthURL":"https://login.live.com/login.srf?username=user%40outlook.com","FederationBrandName":"Windows Live","AuthNForwardType":1,"CloudInstanceName":"microsoftonline.com","CloudInstanceIssuerUri":"urn:federation:MicrosoftOnline"}"#;

    #[test]
    fn work_account_is_titled_after_the_tenant() {
        assert_eq!(title(WORK).as_deref(), Some("OneDrive - Example AS"));
    }

    #[test]
    fn microsoft_account_is_personal() {
        assert_eq!(title(PERSONAL).as_deref(), Some("OneDrive - Personal"));
    }

    #[test]
    fn unknown_domain_has_no_title() {
        assert_eq!(title(r#"{"State":5,"Login":"x@nowhere.invalid"}"#), None);
        assert_eq!(title("not json"), None);
    }

    #[test]
    fn escapes_are_decoded() {
        let json = r#"{"FederationBrandName":"Blå \"Fjord\" A\/S"}"#;
        assert_eq!(title(json).as_deref(), Some("OneDrive - Blå \"Fjord\" A/S"));
    }
}
