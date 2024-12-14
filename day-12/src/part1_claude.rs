use miette::Result;

struct Region {
    area: usize,
    perimeter: usize,
}

impl Region {
    fn price(&self) -> usize {
        self.area * self.perimeter
    }
}

pub fn process(input: &str) -> Result<String> {
    let grid: Vec<Vec<char>> = input.lines().map(|line| line.chars().collect()).collect();

    let regions = find_regions(&grid);
    let total_price: usize = regions.iter().map(|r| r.price()).sum();

    Ok(total_price.to_string())
}

fn find_regions(grid: &Vec<Vec<char>>) -> Vec<Region> {
    let mut visited = vec![vec![false; grid[0].len()]; grid.len()];
    let mut regions = Vec::new();

    for y in 0..grid.len() {
        for x in 0..grid[0].len() {
            if !visited[y][x] {
                let mut area = 0;
                let mut perimeter = 0;
                flood_fill(
                    grid,
                    &mut visited,
                    x,
                    y,
                    grid[y][x],
                    &mut area,
                    &mut perimeter,
                );
                if area > 0 {
                    regions.push(Region { area, perimeter });
                }
            }
        }
    }
    regions
}

fn flood_fill(
    grid: &Vec<Vec<char>>,
    visited: &mut Vec<Vec<bool>>,
    x: usize,
    y: usize,
    target: char,
    area: &mut usize,
    perimeter: &mut usize,
) {
    if y >= grid.len() || x >= grid[0].len() || visited[y][x] || grid[y][x] != target {
        return;
    }

    visited[y][x] = true;
    *area += 1;

    // Count exposed edges
    for (dx, dy) in [(0, 1), (1, 0), (0, -1), (-1, 0)] {
        let nx = x as i32 + dx;
        let ny = y as i32 + dy;

        if nx < 0
            || ny < 0
            || nx >= grid[0].len() as i32
            || ny >= grid.len() as i32
            || grid[ny as usize][nx as usize] != target
        {
            *perimeter += 1;
        }
    }

    // Recurse to neighbors
    for (dx, dy) in [(0, 1), (1, 0), (0, -1), (-1, 0)] {
        let nx = (x as i32 + dx) as usize;
        let ny = (y as i32 + dy) as usize;
        flood_fill(grid, visited, nx, ny, target, area, perimeter);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "RRRRIICCFF
RRRRIICCCF
VVRRRCCFFF
VVRCCCJFFF
VVVVCJJCFE
VVIVCCJJEE
VVIIICJJEE
MIIIIIJJEE
MIIISIJEEE
MMMISSJEEE";
        assert_eq!("1930", process(input)?);
        Ok(())
    }
}
