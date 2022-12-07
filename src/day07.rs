use anyhow::Context;
use std::fmt::Display;
use std::str::FromStr;

const INPUT: &str = include_str!("../inputs/day07.input");

fn main() -> anyhow::Result<()> {
    // PART 1 - 1 hour 26 minutes 53 seconds
    let part_1_solution = calculate_sum_of_directories_with_total_size_at_most(INPUT, 100_000)?;
    println!("part_1_solution: {part_1_solution}");

    Ok(())
}

fn calculate_sum_of_directories_with_total_size_at_most(
    input: &str,
    total_size_at_most: usize,
) -> anyhow::Result<usize> {
    let derived_filesystem = CommandHistory::from_str(input)?
        .derive_filesystem()?
        .ok_or_else(|| anyhow::anyhow!("No filesystem found."))?;
    fn calculate_all_directory_sizes(e: &FilesystemElement) -> Vec<usize> {
        match e {
            FilesystemElement::Directory { children, .. } => {
                let mut output = Vec::new();
                output.push(e.size());
                output.extend_from_slice(
                    children
                        .iter()
                        .filter(|ee| ee.is_directory())
                        .flat_map(calculate_all_directory_sizes)
                        .collect::<Vec<_>>()
                        .as_slice(),
                );
                output
            }
            FilesystemElement::File { .. } => Vec::new(),
        }
    }
    let mmm = calculate_all_directory_sizes(&derived_filesystem.0)
        .iter()
        .filter(|aa| **aa <= total_size_at_most)
        .sum::<usize>();
    Ok(mmm)
}

struct CommandHistory(Vec<ExecutedCommand>);

impl CommandHistory {
    fn derive_filesystem(&self) -> anyhow::Result<Option<Filesystem>> {
        let starting_directory_name = match self.0.first() {
            Some(ExecutedCommand {
                command: Command::ChangeDirectory { target },
                ..
            }) => target.to_owned(),
            Some(ExecutedCommand {
                command: Command::ListDirectoryContents,
                ..
            }) => {
                return Err(anyhow::anyhow!(
                    "First executed command is no ChangeDirectory command, which is needed."
                ))
            }
            None => return Ok(None),
        };

        let mut filesystem = Filesystem(FilesystemElement::directory(starting_directory_name));
        let mut navigation_done = vec![filesystem.0.name().to_owned()];
        let mut current_filesystem_element: &mut FilesystemElement = &mut filesystem.0;
        for (executed_index, executed_command) in self.0.iter().enumerate().skip(1) {
            match &executed_command.command {
                Command::ChangeDirectory { target } => {
                    let current_filesystem_element_for_error =
                        format!("{current_filesystem_element:?}");
                    let target_element = if target == ".." {
                        navigation_done
                            .pop()
                            .ok_or_else(|| anyhow::anyhow!("Cannot go into parent directory, because there is none for \"{current_filesystem_element_for_error}\" during executed command #{executed_index}."))?;
                        filesystem
                            .get_element_by_path_mut(&navigation_done)
                            .with_context(|| format!("while getting element at path {navigation_done:?} during executed command #{executed_index}"))?
                            .ok_or_else(|| anyhow::anyhow!("There is no element at path {navigation_done:?} during executed command #{executed_index}"))?
                    } else {
                        let target_element = current_filesystem_element
                            .get_child_by_name_mut(target)
                            .with_context(|| anyhow::anyhow!("with executed command #{executed_index}"))?
                            .ok_or_else(|| anyhow::anyhow!("Did not find child \"{target}\" in {current_filesystem_element_for_error} during executed command #{executed_index}."))?;
                        navigation_done.push(target_element.name().to_owned());
                        target_element
                    };
                    current_filesystem_element = match target_element {
                        FilesystemElement::File { name, .. } => {
                            return Err(anyhow::anyhow!(
                                "Can not change directory into file \"{name}\"."
                            ))
                        }
                        FilesystemElement::Directory { .. } => target_element,
                    };
                }
                Command::ListDirectoryContents => {
                    let children = executed_command
                        .output_lines
                        .iter()
                        .enumerate()
                        .map(|(line_index, output_line)| output_line
                            .split(' ')
                            .collect::<Vec<_>>()
                            .try_into()
                            .map_err(|vec: Vec<_>| anyhow::anyhow!("Could not split line #{line_index} of executed command #{executed_index} into two parts.")))
                        .collect::<Result<Vec<[&str; 2]>, _>>()?
                        .into_iter()
                        .enumerate()
                        .map(|(line_index, line_components)| if line_components[0] == "dir" {
                            Ok(FilesystemElement::directory(line_components[1].to_owned()))
                        } else {
                            line_components[0]
                                .parse::<usize>()
                                .with_context(|| anyhow::anyhow!("while parsing number of line #{line_index} \"{line_components:?}\" in executed command #{executed_index}"))
                                .map(|file_size| FilesystemElement::file(line_components[1].to_owned(), file_size))
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    current_filesystem_element.add_children(children)?;
                }
            }
        }
        Ok(Some(filesystem))
    }
}

impl FromStr for CommandHistory {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            s.split('$')
                .skip(1)
                .enumerate()
                .map(|(index, executed_command_string)| {
                    ExecutedCommand::from_str(executed_command_string)
                        .with_context(|| format!("for executed command #{index}"))
                })
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }
}

struct ExecutedCommand {
    command: Command,
    output_lines: Vec<String>,
}

impl FromStr for ExecutedCommand {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let command = lines
            .next()
            .map(str::trim)
            .map(Command::from_str)
            .ok_or_else(|| anyhow::anyhow!("Executed command did not have first line."))?
            .with_context(|| format!("for executed command \"{s}\""))?;
        Ok(Self {
            command,
            output_lines: lines.map(str::to_owned).collect(),
        })
    }
}

enum Command {
    ChangeDirectory { target: String },
    ListDirectoryContents,
}

impl FromStr for Command {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(target) = s.strip_prefix("cd ") {
            Ok(Self::ChangeDirectory {
                target: target.to_owned(),
            })
        } else if s == "ls" {
            Ok(Self::ListDirectoryContents)
        } else {
            Err(anyhow::anyhow!("Did not recognize \"{s}\"."))
        }
    }
}

struct Filesystem(FilesystemElement);

impl Filesystem {
    fn get_element_by_path_mut(
        &mut self,
        path: &[String],
    ) -> anyhow::Result<Option<&mut FilesystemElement>> {
        path.first()
            .map(|top_level| {
                if top_level.eq(self.0.name()) {
                    let mut current_element: &mut FilesystemElement = &mut self.0;
                    for path_element in path.iter().skip(1) {
                        current_element =
                            match current_element.get_child_by_name_mut(path_element)? {
                                None => return Ok(None),
                                Some(child) => child,
                            };
                    }
                    Ok(Some(current_element))
                } else {
                    Ok(None)
                }
            })
            .transpose()
            .map(Option::flatten)
    }
}

impl Display for Filesystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
enum FilesystemElement {
    Directory {
        name: String,
        children: Vec<FilesystemElement>,
    },
    File {
        name: String,
        size: usize,
    },
}

impl FilesystemElement {
    fn directory(name: String) -> Self {
        Self::Directory {
            name,
            children: Vec::new(),
        }
    }

    fn file(name: String, size: usize) -> Self {
        Self::File { name, size }
    }

    fn name(&self) -> &str {
        match self {
            FilesystemElement::Directory { name, .. } => name.as_str(),
            FilesystemElement::File { name, .. } => name.as_str(),
        }
    }

    fn size(&self) -> usize {
        match self {
            FilesystemElement::Directory { children, .. } => {
                children.iter().map(FilesystemElement::size).sum::<usize>()
            }
            FilesystemElement::File { size, .. } => *size,
        }
    }

    fn is_directory(&self) -> bool {
        match self {
            FilesystemElement::Directory { .. } => true,
            FilesystemElement::File { .. } => false,
        }
    }

    fn is_file(&self) -> bool {
        match self {
            FilesystemElement::Directory { .. } => false,
            FilesystemElement::File { .. } => true,
        }
    }

    fn add_children(&mut self, children: Vec<FilesystemElement>) -> anyhow::Result<()> {
        match self {
            FilesystemElement::Directory {
                children: self_children,
                ..
            } => {
                self_children.extend(children);
                Ok(())
            }
            FilesystemElement::File { .. } => {
                Err(anyhow::anyhow!("Cannot add children to a file."))
            }
        }
    }

    fn get_child_by_name_mut(
        &mut self,
        name: &str,
    ) -> anyhow::Result<Option<&mut FilesystemElement>> {
        match self {
            FilesystemElement::Directory { children, .. } => {
                Ok(children.iter_mut().find(|child| child.name() == name))
            }
            FilesystemElement::File { .. } => {
                Err(anyhow::anyhow!("A file does not have children."))
            }
        }
    }
}

impl Display for FilesystemElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FilesystemElement::Directory { name, children } => {
                write!(f, "- {name} (dir)")?;
                for child in children {
                    let child_display = format!("{child}")
                        .lines()
                        .map(|line| format!("\n  {line}"))
                        .collect::<String>();
                    write!(f, "{child_display}")?;
                }
                Ok(())
            }
            FilesystemElement::File { name, size } => write!(f, "- {name} (file, size={size})"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = "$ cd /
$ ls
dir a
14848514 b.txt
8504156 c.dat
dir d
$ cd a
$ ls
dir e
29116 f
2557 g
62596 h.lst
$ cd e
$ ls
584 i
$ cd ..
$ cd ..
$ cd d
$ ls
4060174 j
8033020 d.log
5626152 d.ext
7214296 k";

    #[test]
    fn test_part_1_default() -> anyhow::Result<()> {
        // Act
        let sum_of_directories_with_total_size_at_most_100_000 =
            calculate_sum_of_directories_with_total_size_at_most(TEST_INPUT, 100_000)?;

        // Assert
        assert_eq!(sum_of_directories_with_total_size_at_most_100_000, 95_437);

        Ok(())
    }

    #[test]
    fn test_part_1_visual_representation() -> anyhow::Result<()> {
        // Arrange
        const TEST_VISUAL_REPRESENTATION: &str = "- / (dir)
  - a (dir)
    - e (dir)
      - i (file, size=584)
    - f (file, size=29116)
    - g (file, size=2557)
    - h.lst (file, size=62596)
  - b.txt (file, size=14848514)
  - c.dat (file, size=8504156)
  - d (dir)
    - j (file, size=4060174)
    - d.log (file, size=8033020)
    - d.ext (file, size=5626152)
    - k (file, size=7214296)";

        // Act
        let derived_filesystem = CommandHistory::from_str(TEST_INPUT)?
            .derive_filesystem()?
            .ok_or_else(|| anyhow::anyhow!("No filesystem found."))?;

        // Assert
        println!("{TEST_VISUAL_REPRESENTATION}");
        println!("{derived_filesystem}");
        assert_eq!(
            format!("{derived_filesystem}").as_str(),
            TEST_VISUAL_REPRESENTATION
        );

        Ok(())
    }
}
