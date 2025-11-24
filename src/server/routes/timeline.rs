use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Html;

use crate::presentation::templates::TimelineTemplate;
use crate::presentation::views::{TimelineEventDetailView, TimelineEventView, TimelineMonthView};
use crate::server::errors::{AppError, map_app_error};
use crate::server::routes::render_html;
use crate::server::server::AppState;

pub(crate) async fn timeline_page(
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let events = state
        .timeline_repo
        .list_all()
        .await
        .map_err(|err| map_app_error(AppError::from(err)))?;

    let mut months: Vec<TimelineMonthView> = Vec::new();

    for event in events {
        let occurred_at = event.occurred_at;
        let anchor = occurred_at.format("%Y-%m").to_string();
        let heading = occurred_at.format("%B %Y").to_string();

        let entity_id = event.entity_id.clone();
        let kind_label = match event.entity_type.as_str() {
            "roaster" => "Roaster Added",
            "roast" => "Roast Added",
            _ => "Event",
        };
        let link = match event.entity_type.as_str() {
            "roaster" => format!("/roasters/{entity_id}"),
            "roast" => format!("/roasts/{entity_id}"),
            _ => String::from("#"),
        };

        let mut details: Vec<TimelineEventDetailView> = event
            .details
            .into_iter()
            .map(|detail| TimelineEventDetailView {
                label: detail.label,
                value: detail.value,
            })
            .collect();

        let external_link = if let Some(index) = details
            .iter()
            .position(|detail| detail.label.eq_ignore_ascii_case("homepage"))
        {
            let value = details.remove(index).value;
            let trimmed = value.trim();
            if trimmed.is_empty() || trimmed == "—" {
                None
            } else {
                Some(trimmed.to_string())
            }
        } else {
            None
        };

        let tasting_notes = if event.entity_type == "roast" {
            let notes = event
                .tasting_notes
                .clone()
                .into_iter()
                .flat_map(|note| {
                    note.split(|ch| ch == ',' || ch == '\n')
                        .map(|segment| segment.trim())
                        .filter(|segment| !segment.is_empty())
                        .map(|segment| segment.to_string())
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            Some(notes)
        } else {
            None
        };

        let view = TimelineEventView {
            id: event.id,
            kind_label,
            badge_class: "bg-amber-200 text-amber-800",
            accent_class: "bg-amber-600",
            card_border_class: "border-amber-200 bg-amber-50/80",
            title_class: "text-amber-800",
            date_label: occurred_at.format("%B %d, %Y").to_string(),
            time_label: Some(occurred_at.format("%H:%M UTC").to_string()),
            iso_timestamp: occurred_at.to_rfc3339(),
            title: event.title,
            link,
            external_link,
            details,
            tasting_notes,
        };

        if let Some(last) = months.last_mut() {
            if last.anchor == anchor {
                last.events.push(view);
                continue;
            }
        }

        months.push(TimelineMonthView {
            anchor,
            heading,
            events: vec![view],
        });
    }

    let template = TimelineTemplate {
        nav_active: "timeline",
        months,
    };

    render_html(template)
}
