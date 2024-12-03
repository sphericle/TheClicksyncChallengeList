use maud::{html, Markup};
use pointercrate_core_pages::{head::HeadLike, PageFragment};

pub fn home_page() -> PageFragment {
    PageFragment::new("The CSCL", "hi")
        .module("/static/demonlist/js/modules/statsviewer.js")
        .module("/static/demonlist/js/statsviewer/nation.js")
        .stylesheet("/static/demonlist/css/statsviewer.css")
        .stylesheet("/static/core/css/sidebar.css")
        .body(home_page_html())
}

fn home_page_html() -> Markup {
    html! {
        div.panel.fade {
            h1.underlined.pad {
                "The Clicksync Challenge List"
            }

            p {
                "Welcome to the CSCL!"
            }
        }
    }
}
