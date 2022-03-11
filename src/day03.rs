use anyhow::{anyhow, Result};
use core::fmt;
use std::error;
use std::fs;
use std::str::FromStr;

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

impl Line {
    fn contains(&self, pt: Point) -> bool {
        return is_value_in_range(self.p1.x, self.p2.x, pt.x)
            && is_value_in_range(self.p1.y, self.p2.y, pt.y);
    }

    // Returns manhattan distance between two points of line
    fn distance(&self) -> u32 {
        return manhattan_distance(self.p1, self.p2) as u32;
    }
}

#[derive(Debug)]
struct Wire {
    segments: Vec<Line>,
}

impl Wire {
    // Returns number of steps required to reach desired point
    fn steps(&self, pt: Point) -> Option<u32> {
        let mut steps = 0u32;
        for line in &self.segments {
            if line.contains(pt) {
                steps += manhattan_distance(line.p1, pt) as u32;
                return Some(steps);
            }
            steps += line.distance();
        }

        None
    }
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

pub fn solve(path: &str) -> Result<()> {
    let input = fs::read_to_string(path)?;

    let wires: std::result::Result<Vec<_>, _> = input.lines().map(|s| Wire::from_str(s)).collect();
    let wires = wires?;

    let min_distance = distance_to_near_wires_intersect(&wires[0], &wires[1], POINT_CENTER)
        .ok_or(anyhow!("No wire intersection"))?;
    println!("answer 1: {}", min_distance);

    let steps = min_steps_to_wires_intersect(&wires[0], &wires[1])
        .ok_or(anyhow!("Couldn't calculate steps to intersection points"))?;
    println!("answer 2: {}", steps);
    Ok(())
}

fn manhattan_distance(pt1: Point, pt2: Point) -> i32 {
    (pt1.x - pt2.x).abs() + (pt1.y - pt2.y).abs()
}

fn distance_to_near_wires_intersect(wire1: &Wire, wire2: &Wire, target_pt: Point) -> Option<i32> {
    let mut min_distance = 0;

    for pt in wires_intersection_points(wire1, wire2) {
        let dist = manhattan_distance(target_pt, pt);
        if min_distance == 0 || min_distance > dist {
            min_distance = dist;
        }
    }

    if min_distance == 0 {
        None
    } else {
        Some(min_distance)
    }
}

fn min_steps_to_wires_intersect(wire1: &Wire, wire2: &Wire) -> Option<u32> {
    let mut min_steps = 0;

    for pt in wires_intersection_points(wire1, wire2) {
        let wire1_steps = wire1.steps(pt).unwrap();
        let wire2_steps = wire2.steps(pt).unwrap();
        let total_steps = wire1_steps + wire2_steps;
        if min_steps == 0 || min_steps > total_steps {
            min_steps = total_steps;
        }
    }

    if min_steps == 0 {
        None
    } else {
        Some(min_steps)
    }
}

fn wires_intersection_points(wire1: &Wire, wire2: &Wire) -> Vec<Point> {
    let mut points = Vec::new();

    // Collect all intersections
    for line1 in &wire1.segments {
        for line2 in &wire2.segments {
            if let Some(point) = straight_lines_intersection(*line1, *line2) {
                points.push(point);
            }
        }
    }

    points
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
    fn test_lowest_wire_distance() {
        let cases = [
            ("R8,U5,L5,D3", "U7,R6,D4,L4", Some(6)),
            (
                "R75,D30,R83,U83,L12,D49,R71,U7,L72",
                "U62,R66,U55,R34,D71,R55,D58,R83",
                Some(159),
            ),
            (
                "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51",
                "U98,R91,D20,R16,D67,R40,U7,R15,U6,R7",
                Some(135),
            ),
        ];

        for (in1, in2, res) in cases {
            let wire1 = in1.parse::<Wire>().unwrap();
            let wire2 = in2.parse::<Wire>().unwrap();
            assert_eq!(
                distance_to_near_wires_intersect(&wire1, &wire2, POINT_CENTER),
                res
            );
        }
    }

    #[test]
    fn test_wire_steps() {
        let wire = Wire::from_str("R8,U5,L5,D3").unwrap();
        assert_eq!(wire.steps(Point { x: 3, y: 3 }), Some(20));
        assert_eq!(wire.steps(Point { x: 6, y: 5 }), Some(15));

        let wire = Wire::from_str("U7,R6,D4,L4").unwrap();
        assert_eq!(wire.steps(Point { x: 3, y: 3 }), Some(20));
        assert_eq!(wire.steps(Point { x: 6, y: 5 }), Some(15));
    }

    #[test]
    fn test_min_wire_steps() {
        let cases = [
            ("R8,U5,L5,D3", "U7,R6,D4,L4", Some(30)),
            (
                "R75,D30,R83,U83,L12,D49,R71,U7,L72",
                "U62,R66,U55,R34,D71,R55,D58,R83",
                Some(610),
            ),
            (
                "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51",
                "U98,R91,D20,R16,D67,R40,U7,R15,U6,R7",
                Some(410),
            ),
        ];

        for (in1, in2, res) in cases {
            let wire1 = in1.parse::<Wire>().unwrap();
            let wire2 = in2.parse::<Wire>().unwrap();
            assert_eq!(min_steps_to_wires_intersect(&wire1, &wire2), res);
        }
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
