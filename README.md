# Advent of Code

Solutions for the [Advent of Code](http://adventofcode.com/)

## Tooling

This was shamelessly stolen from Chris Biscardi who does an amazing job every year doing AoC in Rust.

If you stumble across this, please check out his repos and YouTube (primarily covers Bevy):

- [YouTube](https://www.youtube.com/@chrisbiscardi)

- [Github](https://github.com/ChristopherBiscardi/advent-of-code)

I just wanted his latest setup, so Claude helped with the git wizardry here ...

```bash
# Clone with sparse checkout
git clone --no-checkout --filter=tree:0 https://github.com/ChristopherBiscardi/advent-of-code temp-clone
cd temp-clone
git sparse-checkout set "2024/rust" ".gitignore"
git checkout <commit-hash>   # Replace <commit-hash> with the actual commit ID

# Rest remains the same
mkdir ../advent-of-code-2024
cp -r 2024/rust/* ../advent-of-code-2024/
cp .gitignore ../advent-of-code-2024/
cd ../advent-of-code-2024
git init
git add .
git commit -m "Initial commit: Starting from ChristopherBiscardi's 2024 Rust template"

# Connect to your new remote repository
git remote add origin https://github.com/YOUR_USERNAME/advent-of-code-2024.git
git push -u origin main

# Clean up
cd ..
rm -rf temp-clone
```

NOTE: I couldn't get the cargo script to work for some reason so I just compiled the script and it works as originally intended. :)