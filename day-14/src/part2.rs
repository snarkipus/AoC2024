use std::fmt::Display;

use nom::{
    bytes::complete::tag,
    character::complete::{char, digit1, newline},
    combinator::{map, opt},
    multi::separated_list1,
    sequence::tuple,
    IResult, Parser,
};

use miette::miette;

#[derive(Debug, Clone, PartialEq)]
struct Robot {
    position: Position,
    velocity: Velocity,
}

impl Robot {
    fn new(position: Position, velocity: Velocity) -> Self {
        Self { position, velocity }
    }

    fn step(&mut self) {
        self.position.0 = (self.position.0 + self.velocity.0).rem_euclid(XDIM as i32);
        self.position.1 = (self.position.1 + self.velocity.1).rem_euclid(YDIM as i32);
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Cell {
    robots: Option<Vec<Robot>>,
    position: Position,
}

impl Cell {
    fn new(position: Position) -> Self {
        Self {
            robots: None,
            position,
        }
    }

    fn count(&self) -> usize {
        match &self.robots {
            Some(robots) => robots.len(),
            None => 0,
        }
    }

    fn clear(&mut self) {
        self.robots = None;
    }
}

#[derive(Debug)]
struct GridView<'a> {
    data: &'a [Vec<Cell>],
    x_start: usize,
    x_end: usize,
    y_start: usize,
    y_end: usize,
}

impl GridView<'_> {
    fn count_robots(&self) -> usize {
        self.data
            .iter()
            .skip(self.y_start)
            .take(self.y_end - self.y_start)
            .map(|row| {
                row.iter()
                    .skip(self.x_start)
                    .take(self.x_end - self.x_start)
                    .map(|cell| cell.count())
                    .sum::<usize>()
            })
            .sum()
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Grid(Vec<Vec<Cell>>);

impl Grid {
    fn quadrants(&self) -> Vec<GridView> {
        let xmid = XDIM / 2;
        let ymid = YDIM / 2;

        vec![
            GridView {
                data: self.0.as_slice(),
                x_start: 0,
                x_end: xmid,
                y_start: 0,
                y_end: ymid,
            },
            GridView {
                data: self.0.as_slice(),
                x_start: xmid + 1,
                x_end: XDIM,
                y_start: 0,
                y_end: ymid,
            },
            GridView {
                data: self.0.as_slice(),
                x_start: 0,
                x_end: xmid,
                y_start: ymid + 1,
                y_end: YDIM,
            },
            GridView {
                data: self.0.as_slice(),
                x_start: xmid + 1,
                x_end: XDIM,
                y_start: ymid + 1,
                y_end: YDIM,
            },
        ]
    }

    fn clear(&mut self) {
        for row in self.0.iter_mut() {
            for cell in row.iter_mut() {
                cell.clear();
            }
        }
    }
}

impl Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.0.iter() {
            for cell in row.iter() {
                match &cell.robots {
                    Some(robots) => {
                        if robots.len() > 1 {
                            write!(f, "X")?;
                        } else {
                            write!(f, "#")?;
                        }
                    }
                    None => write!(f, ".")?,
                }
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

const XDIM: usize = 101;
const YDIM: usize = 103;

// const XDIM: usize = 11;
// const YDIM: usize = 7;

const TICKS: usize = 1000;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, mut robots) =
        parse_robots(input).map_err(|e| miette!("Failed to parse input: {}", e))?;

    let mut grid: Grid = Grid(Vec::with_capacity(YDIM));

    for y in 0..YDIM {
        let mut row = Vec::<Cell>::with_capacity(XDIM);
        for x in 0..XDIM {
            row.push(Cell::new((x as i32, y as i32)));
        }
        grid.0.push(row);
    }

    for robot in robots.iter() {
        let (x, y) = robot.position;
        let cell = &mut grid.0[y as usize][x as usize];
        match &mut cell.robots {
            Some(robots) => robots.push(robot.clone()),
            None => cell.robots = Some(vec![robot.clone()]),
        }
    }

    walk_robots(&mut robots, TICKS, &mut grid)?;

    let quadrants = grid.quadrants();

    let robot_count: usize = quadrants
        .iter()
        .fold(1, |acc, quadrant| acc * quadrant.count_robots());

    Ok(robot_count.to_string())
}

fn walk_robots(robots: &mut [Robot], ticks: usize, grid: &mut Grid) -> miette::Result<()> {
    (1..=ticks).for_each(|tick| {
        robots.iter_mut().for_each(|robot| {
            robot.step();
        });
        
        println!("time: {tick}\n{grid}");
        
        grid.clear();
        for robot in robots.iter() {
            let (x, y) = robot.position;
            let cell = &mut grid.0[y as usize][x as usize];
            match &mut cell.robots {
                Some(robots) => robots.push(robot.clone()),
                None => cell.robots = Some(vec![robot.clone()]),
            }
        }
    });

    Ok(())
}

// region: nom parser
type Position = (i32, i32);
type Velocity = (i32, i32);

fn parse_signed_digit(input: &str) -> IResult<&str, i32> {
    let (input, sign) = map(opt(char('-')), |minus| match minus {
        Some(_) => -1,
        None => 1,
    })(input)?;
    let (input, digit) = digit1(input)?;

    Ok((input, sign * digit.parse::<i32>().unwrap()))
}

fn parse_numbers(input: &str) -> IResult<&str, (i32, i32)> {
    let (input, (x, _, y)) = tuple((parse_signed_digit, tag(","), parse_signed_digit))(input)?;

    Ok((input, (x, y)))
}

fn parse_line(input: &str) -> IResult<&str, (Position, Velocity)> {
    let (input, (_, position, _, velocity)) =
        tuple((tag("p="), parse_numbers, tag(" v="), parse_numbers))(input)?;

    Ok((input, (position, velocity)))
}

fn parse_robots(input: &str) -> IResult<&str, Vec<Robot>> {
    let (input, output) =
        separated_list1(newline, parse_line.map(|(p, v)| Robot::new(p, v)))(input)?;

    Ok((input, output))
}
// endregion

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "\
p=0,4 v=3,-3
p=6,3 v=-1,-3
p=10,3 v=-1,2
p=2,0 v=2,-1
p=0,0 v=1,3
p=3,0 v=-2,-2
p=7,6 v=-1,-3
p=3,0 v=-1,-2
p=9,3 v=2,3
p=7,3 v=-1,2
p=2,4 v=2,-3
p=9,5 v=-3,-3";
        assert_eq!("12", process(input)?);
        Ok(())
    }

    #[test]
    fn test_parse_robots() -> miette::Result<()> {
        let input = "\
p=0,4 v=3,-3
p=6,3 v=-1,-3
p=10,3 v=-1,2
p=2,0 v=2,-1
p=0,0 v=1,3
p=3,0 v=-2,-2
p=7,6 v=-1,-3
p=3,0 v=-1,-2
p=9,3 v=2,3
p=7,3 v=-1,2
p=2,4 v=2,-3
p=9,5 v=-3,-3";

        let (_, mut robots) =
            parse_robots(input).map_err(|e| miette!("Failed to parse input: {}", e))?;

        dbg!(format!(
            "t0: ({},{})",
            robots[10].position.0, robots[10].position.1
        ));
        robots[10].step();
        dbg!(format!(
            "1s: ({},{})",
            robots[10].position.0, robots[10].position.1
        ));
        robots[10].step();
        dbg!(format!(
            "2s: ({},{})",
            robots[10].position.0, robots[10].position.1
        ));
        robots[10].step();
        dbg!(format!(
            "3s: ({},{})",
            robots[10].position.0, robots[10].position.1
        ));
        robots[10].step();
        dbg!(format!(
            "4s: ({},{})",
            robots[10].position.0, robots[10].position.1
        ));
        robots[10].step();
        dbg!(format!(
            "5s: ({},{})",
            robots[10].position.0, robots[10].position.1
        ));

        Ok(())
    }
}
