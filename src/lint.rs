use std::fmt;

/// A calendar date, kept deliberately simple: no time zones, no time of day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date {
    pub year: u32,
    pub month: u32,
    pub day: u32,
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// A single scheduled card, parsed from one line of a deck file.
#[derive(Debug, Clone, PartialEq)]
pub struct Card {
    pub id: String,
    pub due: Date,
    pub interval: i64,
    pub ease: f64,
    pub reps: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Warning,
    Error,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Severity::Warning => "warning",
            Severity::Error => "error",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    pub line: usize,
    pub severity: Severity,
    pub message: String,
}

// SM-2 defines 1.3 as the floor: below it a card's interval stops growing
// and it never leaves daily review.
const MIN_EASE: f64 = 1.3;
const MAX_SANE_EASE: f64 = 3.5;
const MAX_SANE_INTERVAL_DAYS: i64 = 365 * 20;

pub fn parse_date(s: &str) -> Result<Date, String> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 {
        return Err(format!("date '{s}' is not in YYYY-MM-DD form"));
    }
    let year = parts[0]
        .parse::<u32>()
        .map_err(|_| format!("date '{s}' has a non-numeric year"))?;
    let month = parts[1]
        .parse::<u32>()
        .map_err(|_| format!("date '{s}' has a non-numeric month"))?;
    let day = parts[2]
        .parse::<u32>()
        .map_err(|_| format!("date '{s}' has a non-numeric day"))?;

    if !(1..=12).contains(&month) {
        return Err(format!("month {month} in '{s}' is out of range"));
    }
    let max_day = days_in_month(year, month);
    if day < 1 || day > max_day {
        return Err(format!("day {day} in '{s}' is out of range for that month"));
    }
    Ok(Date { year, month, day })
}

fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Parses one non-empty, non-comment line into a `Card`.
///
/// Expected shape: `<id> due=YYYY-MM-DD interval=<int> ease=<float> reps=<int>`,
/// fields separated by whitespace and order-independent.
pub fn parse_card(line: &str) -> Result<Card, String> {
    let mut fields = line.split_whitespace();
    let id = fields
        .next()
        .ok_or_else(|| "line has no card id".to_string())?
        .to_string();

    let mut due: Option<Date> = None;
    let mut interval: Option<i64> = None;
    let mut ease: Option<f64> = None;
    let mut reps: Option<i64> = None;

    for field in fields {
        let (key, value) = field
            .split_once('=')
            .ok_or_else(|| format!("field '{field}' is missing '='"))?;
        match key {
            "due" => due = Some(parse_date(value)?),
            "interval" => {
                interval = Some(
                    value
                        .parse::<i64>()
                        .map_err(|_| format!("interval '{value}' is not an integer"))?,
                )
            }
            "ease" => {
                ease = Some(
                    value
                        .parse::<f64>()
                        .map_err(|_| format!("ease '{value}' is not a number"))?,
                )
            }
            "reps" => {
                reps = Some(
                    value
                        .parse::<i64>()
                        .map_err(|_| format!("reps '{value}' is not an integer"))?,
                )
            }
            other => return Err(format!("unknown field '{other}'")),
        }
    }

    Ok(Card {
        id,
        due: due.ok_or("card is missing a 'due' field")?,
        interval: interval.ok_or("card is missing an 'interval' field")?,
        ease: ease.ok_or("card is missing an 'ease' field")?,
        reps: reps.ok_or("card is missing a 'reps' field")?,
    })
}

/// Checks a single already-parsed card in isolation. Pure: same card in,
/// same issues out, no knowledge of the rest of the deck.
pub fn check_card(card: &Card) -> Vec<(Severity, String)> {
    let mut issues = Vec::new();

    if card.ease < MIN_EASE {
        issues.push((
            Severity::Error,
            format!(
                "ease {:.2} is below the minimum of {MIN_EASE} (the scheduler will stall this card)",
                card.ease
            ),
        ));
    } else if card.ease > MAX_SANE_EASE {
        issues.push((
            Severity::Warning,
            format!(
                "ease {:.2} is unusually high; reviews will space out very fast",
                card.ease
            ),
        ));
    }

    if card.interval < 0 {
        issues.push((
            Severity::Error,
            format!("interval {} is negative", card.interval),
        ));
    } else if card.interval > MAX_SANE_INTERVAL_DAYS {
        issues.push((
            Severity::Warning,
            format!("interval of {} days is longer than 20 years", card.interval),
        ));
    }

    if card.reps < 0 {
        issues.push((Severity::Error, format!("reps {} is negative", card.reps)));
    }

    if card.reps == 0 && card.interval > 1 {
        issues.push((
            Severity::Warning,
            format!(
                "card has 0 reps but an interval of {} days; a new card should start at 0 or 1",
                card.interval
            ),
        ));
    }

    if card.reps > 0 && card.interval == 0 {
        issues.push((
            Severity::Warning,
            format!(
                "card has {} reps but an interval of 0; it should have grown past the first review",
                card.reps
            ),
        ));
    }

    issues
}

/// Lints a whole deck file's contents and returns every finding, in the
/// order the offending lines appear. Blank lines and lines starting with
/// '#' are skipped. Takes the full source as a string so it stays pure:
/// no file handles, nothing but text in and findings out.
pub fn lint_source(source: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut seen_ids: Vec<(String, usize)> = Vec::new();

    for (idx, raw_line) in source.lines().enumerate() {
        let line_no = idx + 1;
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        match parse_card(line) {
            Ok(card) => {
                if let Some((_, first_line)) = seen_ids.iter().find(|(id, _)| id == &card.id) {
                    findings.push(Finding {
                        line: line_no,
                        severity: Severity::Error,
                        message: format!(
                            "duplicate card id '{}' (first seen on line {first_line})",
                            card.id
                        ),
                    });
                } else {
                    seen_ids.push((card.id.clone(), line_no));
                }

                for (severity, message) in check_card(&card) {
                    findings.push(Finding {
                        line: line_no,
                        severity,
                        message: format!("card '{}': {message}", card.id),
                    });
                }
            }
            Err(err) => findings.push(Finding {
                line: line_no,
                severity: Severity::Error,
                message: err,
            }),
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_valid_date() {
        assert_eq!(
            parse_date("2026-02-28").unwrap(),
            Date {
                year: 2026,
                month: 2,
                day: 28
            }
        );
    }

    #[test]
    fn rejects_february_29_in_a_non_leap_year() {
        assert!(parse_date("2026-02-29").is_err());
    }

    #[test]
    fn accepts_february_29_in_a_leap_year() {
        assert!(parse_date("2024-02-29").is_ok());
    }

    #[test]
    fn parses_a_well_formed_card() {
        let card = parse_card("card-1 due=2026-09-20 interval=6 ease=2.5 reps=3").unwrap();
        assert_eq!(card.id, "card-1");
        assert_eq!(card.interval, 6);
        assert_eq!(card.reps, 3);
    }

    #[test]
    fn rejects_a_card_missing_a_field() {
        let err = parse_card("card-1 due=2026-09-20 interval=6 ease=2.5").unwrap_err();
        assert!(err.contains("reps"));
    }

    #[test]
    fn flags_ease_below_the_sm2_floor() {
        let card = Card {
            id: "c".into(),
            due: Date {
                year: 2026,
                month: 1,
                day: 1,
            },
            interval: 6,
            ease: 1.1,
            reps: 3,
        };
        let issues = check_card(&card);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].0, Severity::Error);
    }

    #[test]
    fn flags_duplicate_ids_across_the_file() {
        let source = "\
card-1 due=2026-01-01 interval=1 ease=2.5 reps=1
card-1 due=2026-01-02 interval=1 ease=2.5 reps=1
";
        let findings = lint_source(source);
        assert!(findings
            .iter()
            .any(|f| f.severity == Severity::Error && f.message.contains("duplicate")));
    }

    #[test]
    fn clean_deck_has_no_findings() {
        let source = "card-1 due=2026-01-01 interval=1 ease=2.5 reps=1\n";
        assert!(lint_source(source).is_empty());
    }
}
