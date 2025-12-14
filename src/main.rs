use chrono::Local;
use colored::*;
use serde::Deserialize;
use std::error::Error;
use std::io::{self, Write};
use unicode_width::UnicodeWidthStr;

#[derive(Deserialize, Debug)]
struct WeatherResponse {
    weather: Vec<WeatherDetail>,
    main: MainDetail,
    wind: WindDetail,
    name: String,
}

#[derive(Deserialize, Debug)]
struct WeatherDetail {
    description: String,
}

#[derive(Deserialize, Debug)]
struct MainDetail {
    temp: f64,
    humidity: i32,
}

#[derive(Deserialize, Debug)]
struct WindDetail {
    speed: f64,
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}

fn wait_for_enter() {
    println!(
        "{}",
        "\nDevam etmek için [ENTER] tuşuna basın...".bright_magenta()
    );
    let mut _s = String::new();
    io::stdin().read_line(&mut _s).unwrap();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let predefined_cities = [
        ("İstanbul", "Istanbul"),
        ("Ankara", "Ankara"),
        ("İzmir", "Izmir"),
        ("Londra", "London"),
        ("New York", "New York"),
        ("Moskova", "Moscow"),
        ("Pekin", "Beijing"),
        ("Tokyo", "Tokyo"),
        ("Berlin", "Berlin"),
        ("Paris", "Paris"),
        ("Selanik", "Thessaloniki"),
        ("Bakü", "Baku"),
    ];

    let api_key = "e926dc9c8c8804950a6f0a89aa4eb501";

    loop {
        clear_screen();
        print_header();

        for (i, (display_name, _)) in predefined_cities.iter().enumerate() {
            println!("  [{}] {}", i + 1, display_name);
        }

        println!("  [{}] {}", 99, "🔍 MANUEL ŞEHİR ARAMA".green().bold());
        println!("\n  [{}] {}", 0, "ÇIKIŞ YAP (Exit)".red());
        println!("{}", "========================================".cyan());
        print!("Seçiminiz > ");

        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let selection = input.trim();

        if selection == "0" {
            println!("{}", "Güle güle! 👋".magenta());
            break;
        }

        let city_to_search: String = match selection.parse::<usize>() {
            Ok(num) => {
                if num >= 1 && num <= predefined_cities.len() {
                    predefined_cities[num - 1].1.to_string()
                } else if num == 99 {
                    print!("{}", "\nAranacak Şehri Yazın: ".yellow());
                    io::stdout().flush().unwrap();
                    let mut city_input = String::new();
                    io::stdin().read_line(&mut city_input)?;
                    city_input.trim().to_string()
                } else {
                    println!("{}", "⚠️  Geçersiz numara!".red());
                    wait_for_enter();
                    continue;
                }
            }
            Err(_) => {
                println!("{}", "⚠️  Lütfen bir sayı giriniz!".red());
                wait_for_enter();
                continue;
            }
        };

        if city_to_search.is_empty() {
            continue;
        }

        let url = format!(
            "http://api.openweathermap.org/data/2.5/weather?q={}&units=metric&appid={}&lang=tr",
            city_to_search, api_key
        );

        println!("\n📡 {} için veri çekiliyor...", city_to_search.cyan());

        match reqwest::get(&url).await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<WeatherResponse>().await {
                        Ok(data) => print_weather_table(&data),
                        Err(_) => println!("{}", "❌ Veri formatı hatası!".red()),
                    }
                } else if response.status() == 404 {
                    println!("{}", "❌ Şehir bulunamadı!".red());
                } else if response.status() == 401 {
                    println!("{}", "❌ API Anahtarı yetkisiz (401).".red());
                } else {
                    println!("{} {}", "❌ Hata:".red(), response.status());
                }
            }
            Err(e) => println!("{} {}", "❌ Bağlantı hatası:".red(), e),
        }

        wait_for_enter();
    }

    Ok(())
}

fn print_header() {
    println!("{}", "========================================".cyan());
    println!(
        "   {} ",
        "🌤️  HAVA DURUMU İSTASYONU v10.0 🌤️".yellow().bold()
    );
    println!("{}", "========================================".cyan());
    let now = Local::now();
    println!("📅 Tarih: {}", now.format("%d.%m.%Y").to_string().cyan());
    println!("----------------------------------------\n");
}

fn print_weather_table(data: &WeatherResponse) {
    let temp = data.main.temp;

    let icon = if temp < 10.0 {
        "❄️"
    } else if temp < 25.0 {
        "🌤️"
    } else {
        "🔥"
    };

    let color_func: fn(String) -> ColoredString = if temp < 10.0 {
        |s| s.cyan().bold()
    } else if temp < 25.0 {
        |s| s.green().bold()
    } else {
        |s| s.red().bold()
    };

    let now = Local::now();
    let date_str = now.format("%d.%m.%Y").to_string();
    let city_name = data.name.to_uppercase();
    let status_str = data.weather[0].description.clone();

    let temp_val_str = format!("{:.1} °C", temp);

    let humidity_str = format!("% {}", data.main.humidity);
    let wind_str = format!("{:.1} m/s", data.wind.speed);

    let table_inner_width = 38;

    println!("\n╔{}╗", "═".repeat(table_inner_width + 2));

    print_centered_row_safe(&city_name, table_inner_width, |s| s.yellow().bold());
    println!("╠{}╣", "═".repeat(table_inner_width + 2));

    print_row_standard("📅 Tarih", &date_str, table_inner_width, |s| s.cyan());
    print_row_standard("📝 Durum", &status_str, table_inner_width, |s| s.white());

    print_row_emoji_fix(
        "🌡️  Sıcaklık",
        &temp_val_str,
        icon,
        table_inner_width,
        color_func,
    );

    print_row_standard("💧 Nem", &humidity_str, table_inner_width, |s| s.white());
    print_row_standard("💨 Rüzgar", &wind_str, table_inner_width, |s| s.white());

    println!("╚{}╝", "═".repeat(table_inner_width + 2));
}

fn print_row_standard<F>(label: &str, value_text: &str, total_width: usize, colorizer: F)
where
    F: Fn(String) -> ColoredString,
{
    let label_width = UnicodeWidthStr::width(label);
    let text_width = UnicodeWidthStr::width(value_text);
    let separator = ": ";

    let used = label_width + 2 + text_width;
    let padding = if total_width > used {
        total_width - used
    } else {
        0
    };

    let target_label_area = 13;
    let label_padding = if target_label_area > label_width {
        target_label_area - label_width
    } else {
        0
    };
    let right_padding = if padding > label_padding {
        padding - label_padding
    } else {
        0
    };

    println!(
        "║ {}{}{}{}{} ║",
        label,
        " ".repeat(label_padding),
        separator,
        colorizer(value_text.to_string()),
        " ".repeat(right_padding)
    );
}

fn print_row_emoji_fix<F>(
    label: &str,
    value_text: &str,
    emoji: &str,
    total_width: usize,
    colorizer: F,
) where
    F: Fn(String) -> ColoredString,
{
    let label_width = UnicodeWidthStr::width(label);
    let text_width = UnicodeWidthStr::width(value_text);
    let separator = ": ";

    let emoji_vis_width = 1;

    let used = label_width + 2 + text_width + emoji_vis_width;

    let padding = if total_width > used {
        total_width - used
    } else {
        0
    };

    let target_label_area = 13;
    let label_padding = if target_label_area > label_width {
        target_label_area - label_width
    } else {
        0
    };
    let right_padding = if padding > label_padding {
        padding - label_padding
    } else {
        0
    };

    println!(
        "║ {}{}{}{} {}{} ║",
        label,
        " ".repeat(label_padding),
        separator,
        colorizer(value_text.to_string()),
        emoji,
        " ".repeat(right_padding)
    );
}

fn print_centered_row_safe<F>(text: &str, width: usize, colorizer: F)
where
    F: Fn(String) -> ColoredString,
{
    let text_len = UnicodeWidthStr::width(text);
    let padding = if width > text_len {
        width - text_len
    } else {
        0
    };
    let left_pad = padding / 2;
    let right_pad = padding - left_pad;

    println!(
        "║ {}{}{} ║",
        " ".repeat(left_pad),
        colorizer(text.to_string()),
        " ".repeat(right_pad)
    );
}
