use actix_web::cookie::{time::Duration as CookieDuration, Cookie, SameSite};
use galahad_core::SessionToken;

pub(crate) fn session_cookie_for(name: &str, token: &SessionToken) -> Cookie<'static> {
    Cookie::build(name.to_owned(), token.as_str().to_owned())
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .finish()
}

pub(crate) fn expired_session_cookie_for(name: &str) -> Cookie<'static> {
    Cookie::build(name.to_owned(), String::new())
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(CookieDuration::seconds(0))
        .finish()
}

#[cfg(test)]
mod tests {
    use super::session_cookie_for;

    #[test]
    fn session_cookie_is_http_only() {
        let cookie = session_cookie_for(
            "galahad_session",
            &galahad_core::SessionToken::from("token"),
        );

        assert_eq!(cookie.name(), "galahad_session");
        assert_eq!(cookie.value(), "token");
        assert_eq!(cookie.http_only(), Some(true));
    }
}
