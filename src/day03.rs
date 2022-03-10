use core::fmt;
use std::error;
use std::fs;
use std::str::FromStr;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Line {
    p1: Point,
    p2: Point,
}

#[derive(Debug)]
struct Wire {
    segments: Vec<Line>,
}

#[derive(Debug, Clone)]
struct ParseWireEror;

impl fmt::Display for ParseWireEror {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed to parse wire string")
    }
}

impl error::Error for ParseWireEror {}

impl FromStr for Wire {
    type Err = ParseWireEror;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let items: Vec<&str> = s.split(',').map(|s| s.trim()).collect();
        let mut p1 = Point { x: 0, y: 0 };
        let mut segments = Vec::new();
        for item in items {
            if item.chars().count() < 2 {
                return Err(ParseWireEror);
            }

            let (dir, count) = item.split_at(1);

            let count = count.parse::<i32>().or(Err(ParseWireEror))?;

            let p2 = match dir.chars().nth(0).unwrap() {
                'R' => Ok(Point {
                    x: p1.x + count,
                    ..p1
                }),
                'L' => Ok(Point {
                    x: p1.x - count,
                    ..p1
                }),
                'U' => Ok(Point {
                    x: p1.x,
                    y: p1.y + count,
                }),
                'D' => Ok(Point {
                    x: p1.x,
                    y: p1.y - count,
                }),
                _ => Err(ParseWireEror),
            }?;

            segments.push(Line { p1, p2 });
            p1 = p2;
        }

        Ok(Wire { segments })
    }
}

const POINT_CENTER: Point = Point { x: 0, y: 0 };

pub(crate) fn day03(path: &str) -> Result<()> {
    let input = fs::read_to_string(path)?;

    let wires: std::result::Result<Vec<_>, _> = input.lines().map(|s| Wire::from_str(s)).collect();
    let wires = wires?;

    let point =
        find_wires_intersection_point(&wires[0], &wires[1]).ok_or("No wire intersection")?;

    println!("answer: {}", manhattan_distance(POINT_CENTER, point));
    Ok(())
}

fn manhattan_distance(pt1: Point, pt2: Point) -> i32 {
    (pt1.x - pt2.x).abs() + (pt1.y - pt2.y).abs()
}

fn find_wires_intersection_point(wire1: &Wire, wire2: &Wire) -> Option<Point> {
    let mut pt_distance = 0;
    let mut pt = None;

    // Collect all intersections
    for line1 in &wire1.segments {
        for line2 in &wire2.segments {
            if let Some(point) = straight_lines_intersection(*line1, *line2) {
                let dist = manhattan_distance(POINT_CENTER, point);
                if pt_distance == 0 || pt_distance > dist {
                    pt_distance = dist;
                    pt = Some(point);
                }
            }
        }
    }

    pt
}

fn is_value_in_range(start: i32, end: i32, val: i32) -> bool {
    if start > end {
        return is_value_in_range(end, start, val);
    }
    return start <= val && val <= end;
}

fn straight_lines_intersection(l1: Line, l2: Line) -> Option<Point> {
    return match (l1.p1, l1.p2, l2.p1, l2.p2) {
        (p1, p2, p3, p4) if p1.x == p2.x && p3.x != p4.x => {
            // l1 is vertical and l2 is horizontal
            if is_value_in_range(p1.y, p2.y, p3.y) && is_value_in_range(p3.x, p4.x, p1.x) {
                // println!("V: {:?} {:?} | {:?} {:?}", p1, p2, p3, p4);
                Some(Point { x: p1.x, ..p3 })
            } else {
                None
            }
        }
        (p1, p2, p3, p4) if p1.y == p2.y && p3.y != p4.y => {
            // l1 is horizontal and l2 is vectical
            if is_value_in_range(p1.x, p2.x, p3.x) && is_value_in_range(p3.y, p4.y, p1.y) {
                // println!("H: {:?} {:?} | {:?} {:?}", p1, p2, p3, p4);
                Some(Point { x: p3.x, ..p1 })
            } else {
                None
            }
        }
        _ => None,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_line(p1: (i32, i32), p2: (i32, i32)) -> Line {
        Line {
            p1: Point { x: p1.0, y: p1.1 },
            p2: Point { x: p2.0, y: p2.1 },
        }
    }

    #[test]
    fn test_parse_wire() {
        let wire = Wire::from_str("R8, U5, L5, D3");
        assert_eq!(wire.is_ok(), true);
        let wire = wire.unwrap();
        assert_eq!(
            wire.segments,
            vec![
                build_line((0, 0), (8, 0)),
                build_line((8, 0), (8, 5)),
                build_line((8, 5), (3, 5)),
                build_line((3, 5), (3, 2))
            ]
        );

        let wire = Wire::from_str("U7,R6,D4,L4");
        assert_eq!(wire.is_ok(), true);
        let wire = wire.unwrap();
        assert_eq!(
            wire.segments,
            vec![
                build_line((0, 0), (0, 7)),
                build_line((0, 7), (6, 7)),
                build_line((6, 7), (6, 3)),
                build_line((6, 3), (2, 3))
            ]
        );
    }

    #[test]
    fn test_wire_intersection() {
        let wire1 = Wire::from_str("R8,U5,L5,D3").unwrap();
        let wire2 = Wire::from_str("U7,R6,D4,L4").unwrap();

        assert_eq!(
            find_wires_intersection_point(&wire1, &wire2),
            Some(Point { x: 3, y: 3 })
        );

        let wire1 = Wire::from_str("R75,D30,R83,U83,L12,D49,R71,U7,L72").unwrap();
        let wire2 = Wire::from_str("U62,R66,U55,R34,D71,R55,D58,R83").unwrap();
        let point = find_wires_intersection_point(&wire1, &wire2);
        assert_eq!(point.is_some(), true);
        let point = point.unwrap();
        assert_eq!(point.x + point.y, 159);

        let wire1 = Wire::from_str("R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51").unwrap();
        let wire2 = Wire::from_str("U98,R91,D20,R16,D67,R40,U7,R15,U6,R7").unwrap();
        let point = find_wires_intersection_point(&wire1, &wire2);
        assert_eq!(point.is_some(), true);
        let point = point.unwrap();
        assert_eq!(point.x + point.y, 135);
    }

    #[test]
    fn test_lines_intersection() {
        let l1 = build_line((3, 5), (3, 3));
        let l2 = build_line((6, 3), (2, 3));
        assert_eq!(
            straight_lines_intersection(l1, l2),
            Some(Point { x: 3, y: 3 })
        );
        assert_eq!(
            straight_lines_intersection(l2, l1),
            Some(Point { x: 3, y: 3 })
        );

        let l1 = build_line((8, 0), (8, 5));
        let l2 = build_line((0, 7), (6, 7));
        assert_eq!(straight_lines_intersection(l1, l2), None);

        let l1 = build_line((8, 5), (3, 5));
        let l2 = build_line((6, 7), (6, 3));
        assert_eq!(
            straight_lines_intersection(l1, l2),
            Some(Point { x: 6, y: 5 })
        );

        let l1 = build_line((0, 0), (8, 0));
        let l2 = build_line((0, 0), (0, 7));
        assert_eq!(
            straight_lines_intersection(l1, l2),
            Some(Point { x: 0, y: 0 })
        );
    }
}