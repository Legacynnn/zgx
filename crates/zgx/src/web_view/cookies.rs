#![allow(missing_docs)]

use super::WebProfile;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WebCookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub expires: Option<f64>,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: Option<String>,
}

pub type CookiesStored = Box<dyn FnOnce(Result<usize, String>)>;
pub type CookieDomains = Box<dyn FnOnce(Result<Vec<(String, usize)>, String>)>;

pub trait WebCookieStore {
    fn set_cookies(&self, profile: &WebProfile, cookies: Vec<WebCookie>, done: CookiesStored);
    fn cookie_domains(&self, profile: &WebProfile, done: CookieDomains);
}

thread_local! {
    static COOKIE_STORE: RefCell<Option<Rc<dyn WebCookieStore>>> = const { RefCell::new(None) };
}

#[doc(hidden)]
pub fn set_cookie_store(store: Rc<dyn WebCookieStore>) {
    COOKIE_STORE.with(|slot| *slot.borrow_mut() = Some(store));
}

fn cookie_store(profile: &WebProfile) -> Result<Rc<dyn WebCookieStore>, String> {
    if *profile == WebProfile::Ephemeral {
        return Err("an ephemeral profile keeps no cookies".into());
    }
    COOKIE_STORE
        .with(|slot| slot.borrow().clone())
        .ok_or_else(|| "cookies are not supported by this platform's web view".into())
}

pub fn set_cookies(
    profile: &WebProfile,
    cookies: Vec<WebCookie>,
    done: impl FnOnce(Result<usize, String>) + 'static,
) {
    match cookie_store(profile) {
        Ok(store) => store.set_cookies(profile, cookies, Box::new(done)),
        Err(error) => done(Err(error)),
    }
}

pub fn cookie_domains(
    profile: &WebProfile,
    done: impl FnOnce(Result<Vec<(String, usize)>, String>) + 'static,
) {
    match cookie_store(profile) {
        Ok(store) => store.cookie_domains(profile, Box::new(done)),
        Err(error) => done(Err(error)),
    }
}

#[doc(hidden)]
pub fn count_cookie_domains(domains: impl IntoIterator<Item = String>) -> Vec<(String, usize)> {
    let mut counts = std::collections::BTreeMap::<String, usize>::new();
    for domain in domains {
        let domain = domain.trim_start_matches('.').to_ascii_lowercase();
        if !domain.is_empty() {
            *counts.entry(domain).or_default() += 1;
        }
    }
    counts.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domains_count_without_their_leading_dot() {
        let counts = count_cookie_domains(
            [".Google.com", "google.com", "mail.google.com", ""].map(String::from),
        );
        assert_eq!(
            counts,
            vec![("google.com".into(), 2), ("mail.google.com".into(), 1)]
        );
    }

    #[test]
    fn without_a_store_cookies_are_not_supported() {
        let answer = Rc::new(RefCell::new(None));
        let kept = answer.clone();
        set_cookies(&WebProfile::Default, Vec::new(), move |result| {
            *kept.borrow_mut() = Some(result)
        });
        let error = answer.borrow_mut().take().unwrap().unwrap_err();
        assert!(error.contains("not supported"), "{error}");
    }
}
