use axum::{response::Html, routing::get, Router};
use crepuscularity_core::TemplateContext;
use crepuscularity_web::render_template_to_html;
use serde::Deserialize;

const TEMPLATE: &str = include_str!("../../weather.crepus");

#[derive(Deserialize)]
struct IpLocation {
    city: String,
    country: String,
    #[allow(dead_code)]
    lat: f64,
    #[allow(dead_code)]
    lon: f64,
}

#[derive(Deserialize)]
struct WttrCurrent {
    #[serde(rename = "temp_C")]
    temp_c: String,
    #[serde(rename = "FeelsLikeC")]
    feels_like_c: String,
    humidity: String,
    #[serde(rename = "windspeedKmph")]
    windspeed_kmph: String,
    #[serde(rename = "weatherDesc")]
    weather_desc: Vec<WttrDesc>,
}

#[derive(Deserialize)]
struct WttrDesc {
    value: String,
}

#[derive(Deserialize)]
struct WttrResponse {
    current_condition: Vec<WttrCurrent>,
}

async fn fetch_weather() -> (String, String, String, String, String, String, String) {
    let (mut city, mut country, mut temp, mut desc, mut feels, mut humidity, mut wind) = (
        "London".to_string(),
        "GB".to_string(),
        "14".to_string(),
        "Partly cloudy".to_string(),
        "12".to_string(),
        "72".to_string(),
        "15".to_string(),
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(4))
        .build()
        .unwrap_or_default();

    if let Ok(resp) = client.get("http://ip-api.com/json/").send().await {
        if let Ok(loc) = resp.json::<IpLocation>().await {
            city = loc.city.clone();
            country = loc.country.clone();

            let url = format!(
                "https://wttr.in/{}?format=j1",
                urlencoding::encode(&city)
            );
            if let Ok(wresp) = client.get(&url).send().await {
                if let Ok(wttr) = wresp.json::<WttrResponse>().await {
                    if let Some(cur) = wttr.current_condition.first() {
                        temp = cur.temp_c.clone();
                        feels = cur.feels_like_c.clone();
                        humidity = cur.humidity.clone();
                        wind = cur.windspeed_kmph.clone();
                        if let Some(d) = cur.weather_desc.first() {
                            desc = d.value.clone();
                        }
                    }
                }
            }
        }
    }

    (city, country, temp, desc, feels, humidity, wind)
}

async fn index() -> Html<String> {
    let (city, country, temp, desc, feels, humidity, wind) = fetch_weather().await;

    let mut ctx = TemplateContext::new();
    ctx.set("city", city.as_str());
    ctx.set("country", country.as_str());
    ctx.set("temp", temp.as_str());
    ctx.set("description", desc.as_str());
    ctx.set("feels_like", feels.as_str());
    ctx.set("humidity", humidity.as_str());
    ctx.set("wind_kmh", wind.as_str());

    let body = render_template_to_html(TEMPLATE, &ctx)
        .unwrap_or_else(|e| format!("<pre style='color:red'>{e}</pre>"));

    Html(format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Weather — {city}</title>
  <script src="https://cdn.tailwindcss.com"></script>
</head>
<body>
{body}
</body>
</html>"#,
        city = city
    ))
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(index));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3002").await.unwrap();
    println!("weather-web-server listening on http://localhost:3002");
    axum::serve(listener, app).await.unwrap();
}
