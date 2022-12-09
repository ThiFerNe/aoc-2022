use std::cell::RefCell;
use std::fmt::Display;
use std::rc::{Rc, Weak};
use std::str::FromStr;

use anyhow::Context;

const INPUT: &str = include_str!("../inputs/day07.input");

fn main() -> anyhow::Result<()> {
    let derived_filesystem = CommandHistory::from_str(INPUT)?
        .derive_filesystem()?
        .ok_or_else(|| anyhow::anyhow!("No filesystem found."))?;

    // PART 1 - 1 hour 26 minutes 53 seconds
    let part_1_solution =
        derived_filesystem.calculate_sum_of_directories_sizes_where_each_size_max(100_000);
    println!("part_1_solution: {part_1_solution}");

    // PART 2 - 10 minutes 10 seconds
    let part_2_solution = derived_filesystem
        .find_directory_size_to_delete_to_free_enough_space(70_000_000, 30_000_000)?;
    println!("part_2_solution: {part_2_solution}");

    Ok(())
}

struct CommandHistory(Vec<ExecutedCommand>);

impl CommandHistory {
    fn derive_filesystem(&self) -> anyhow::Result<Option<Filesystem>> {
        let starting_directory_name = match self.0.first() {
            Some(executed_command) => executed_command
                .command
                .as_change_directory_target()
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "First executed command is no ChangeDirectory command, which is needed."
                    )
                })?
                .to_owned(),
            None => return Ok(None),
        };

        let filesystem = Filesystem::new(FilesystemElement::new_directory(starting_directory_name));

        let mut current_filesystem_element: Rc<RefCell<FilesystemElement>> = filesystem.0.clone();
        for (executed_index, executed_command) in self.0.iter().enumerate().skip(1) {
            match &executed_command.command {
                Command::ChangeDirectory { target } => {
                    if target == ".." {
                        let parent = RefCell::borrow(&current_filesystem_element)
                            .parent()
                            .ok_or_else(|| {
                                anyhow::anyhow!(
                                    "Failed going one up from {:?}.",
                                    RefCell::borrow(&current_filesystem_element)
                                )
                            })?
                            .with_context(|| {
                                format!(
                                    "while trying to go one up from {:?}",
                                    RefCell::borrow(&current_filesystem_element)
                                )
                            })?;
                        current_filesystem_element = parent;
                    } else {
                        let child = RefCell::borrow(&current_filesystem_element)
                            .get_child_by_name(&target)
                            .with_context(|| {
                                format!(
                                    "while trying to get child \"{target}\" from {:?}",
                                    RefCell::borrow(&current_filesystem_element)
                                )
                            })?
                            .ok_or_else(|| {
                                anyhow::anyhow!(
                                    "Did not find a child \"{target}\" from {:?}",
                                    RefCell::borrow(&current_filesystem_element)
                                )
                            })?;
                        current_filesystem_element = child;
                    }
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
                            .map_err(|vec: Vec<_>| anyhow::anyhow!("Could not split line #{line_index} of executed command #{executed_index} into two parts (splitted into {} parts).", vec.len())))
                        .collect::<Result<Vec<[&str; 2]>, _>>()?
                        .into_iter()
                        .enumerate()
                        .map(|(line_index, line_components)| if line_components[0] == "dir" {
                            Ok(FilesystemElement::new_directory(line_components[1].to_owned()))
                        } else {
                            line_components[0]
                                .parse::<usize>()
                                .with_context(|| anyhow::anyhow!("while parsing number of line #{line_index} \"{line_components:?}\" in executed command #{executed_index}"))
                                .map(|file_size| FilesystemElement::new_file(line_components[1].to_owned(), file_size))
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let parent_ref = Rc::downgrade(&current_filesystem_element);
                    current_filesystem_element
                        .borrow_mut()
                        .add_children(children, parent_ref)?;
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

impl Command {
    fn as_change_directory_target(&self) -> Option<&String> {
        match self {
            Command::ChangeDirectory { target } => Some(target),
            Command::ListDirectoryContents => None,
        }
    }
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

struct Filesystem(Rc<RefCell<FilesystemElement>>);

impl Filesystem {
    fn new(filesystem_element: FilesystemElement) -> Self {
        Self(Rc::new(RefCell::new(filesystem_element)))
    }

    fn all_filesystem_elements(&self) -> impl Iterator<Item = Rc<RefCell<FilesystemElement>>> {
        [self.0.clone()]
            .into_iter()
            .chain(RefCell::borrow(&self.0).tree_without_self())
    }

    fn all_directories(&self) -> impl Iterator<Item = Rc<RefCell<FilesystemElement>>> {
        self.all_filesystem_elements()
            .filter(|element| RefCell::borrow(&element).is_directory())
    }

    fn calculate_sum_of_directories_sizes_where_each_size_max(
        &self,
        maximum_size_each: usize,
    ) -> usize {
        self.all_directories()
            .map(|directory| RefCell::borrow(&directory).size())
            .filter(|directory_size| *directory_size <= maximum_size_each)
            .sum::<usize>()
    }

    fn find_directory_size_to_delete_to_free_enough_space(
        &self,
        total_disk_space_available: usize,
        needed_unused_space: usize,
    ) -> anyhow::Result<usize> {
        let amount_to_free =
            RefCell::borrow(&self.0).size() - (total_disk_space_available - needed_unused_space);
        self.find_directory_with_minimum_size_and_at_least_size_of(amount_to_free)
            .ok_or_else(|| anyhow::anyhow!("No directory found, because there might be none."))
            .map(|directory| RefCell::borrow(&directory).size())
    }

    fn find_directory_with_minimum_size_and_at_least_size_of(
        &self,
        at_least_size: usize,
    ) -> Option<Rc<RefCell<FilesystemElement>>> {
        self.all_directories()
            .map(|directory| {
                let directory_size = RefCell::borrow(&directory).size();
                (directory, directory_size)
            })
            .filter(|(_, directory_size)| *directory_size >= at_least_size)
            .reduce(|a, b| b.1.lt(&a.1).then_some(b).unwrap_or(a))
            .map(|(directory, _)| directory)
    }
}

impl Display for Filesystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", RefCell::borrow(&self.0))
    }
}

#[derive(Debug, Clone)]
enum FilesystemElement {
    Directory {
        name: String,
        parent: Option<Weak<RefCell<FilesystemElement>>>,
        children: Vec<Rc<RefCell<FilesystemElement>>>,
    },
    File {
        name: String,
        parent: Option<Weak<RefCell<FilesystemElement>>>,
        size: usize,
    },
}

impl FilesystemElement {
    fn new_directory(name: String) -> Self {
        Self::Directory {
            name,
            parent: None,
            children: Vec::new(),
        }
    }

    fn new_file(name: String, size: usize) -> Self {
        Self::File {
            name,
            parent: None,
            size,
        }
    }

    fn name(&self) -> &str {
        match self {
            FilesystemElement::Directory { name, .. } => name.as_str(),
            FilesystemElement::File { name, .. } => name.as_str(),
        }
    }

    fn parent(&self) -> Option<anyhow::Result<Rc<RefCell<Self>>>> {
        match self {
            Self::Directory { parent, .. } | Self::File { parent, .. } => {
                parent.as_ref().map(|parent| {
                    parent
                        .upgrade()
                        .ok_or_else(|| anyhow::anyhow!("Parent has been destroyed, it seems."))
                })
            }
        }
    }

    fn set_parent(&mut self, new_parent: Weak<RefCell<Self>>) {
        match self {
            FilesystemElement::Directory { parent, .. } => *parent = Some(new_parent),
            FilesystemElement::File { parent, .. } => *parent = Some(new_parent),
        }
    }

    fn size(&self) -> usize {
        match self {
            FilesystemElement::Directory { children, .. } => children
                .iter()
                .map(|child| RefCell::borrow(&child).size())
                .sum::<usize>(),
            FilesystemElement::File { size, .. } => *size,
        }
    }

    fn is_directory(&self) -> bool {
        match self {
            FilesystemElement::Directory { .. } => true,
            FilesystemElement::File { .. } => false,
        }
    }

    #[allow(dead_code)]
    fn is_file(&self) -> bool {
        match self {
            FilesystemElement::Directory { .. } => false,
            FilesystemElement::File { .. } => true,
        }
    }

    fn tree_without_self(&self) -> impl Iterator<Item = Rc<RefCell<Self>>> {
        match self {
            FilesystemElement::Directory { children, .. } => children
                .iter()
                .cloned()
                .flat_map(|child| {
                    [child.clone()]
                        .into_iter()
                        .chain(RefCell::borrow(&child).tree_without_self())
                })
                .collect::<Vec<_>>()
                .into_iter(),
            FilesystemElement::File { .. } => Vec::new().into_iter(),
        }
    }

    fn add_children(
        &mut self,
        children: Vec<Self>,
        parent_ref: Weak<RefCell<Self>>,
    ) -> anyhow::Result<()> {
        match self {
            Self::Directory {
                children: self_children,
                ..
            } => {
                self_children.extend(
                    children
                        .into_iter()
                        .map(|mut child| {
                            child.set_parent(parent_ref.clone());
                            child
                        })
                        .map(RefCell::new)
                        .map(Rc::new),
                );
                Ok(())
            }
            Self::File { .. } => Err(anyhow::anyhow!("Cannot add children to a file.")),
        }
    }

    fn get_child_by_name(
        &self,
        name: &str,
    ) -> anyhow::Result<Option<Rc<RefCell<FilesystemElement>>>> {
        match self {
            FilesystemElement::Directory { children, .. } => Ok(children
                .iter()
                .find(|child| RefCell::borrow(&child).name() == name)
                .cloned()),
            FilesystemElement::File { .. } => {
                Err(anyhow::anyhow!("A file does not have children."))
            }
        }
    }
}

impl Display for FilesystemElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FilesystemElement::Directory { name, children, .. } => {
                write!(f, "- {name} (dir)")?;
                for child in children {
                    let child_display = format!("{}", RefCell::borrow(&child))
                        .lines()
                        .map(|line| format!("\n  {line}"))
                        .collect::<String>();
                    write!(f, "{child_display}")?;
                }
                Ok(())
            }
            FilesystemElement::File { name, size, .. } => write!(f, "- {name} (file, size={size})"),
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
            CommandHistory::from_str(TEST_INPUT)?
                .derive_filesystem()?
                .ok_or_else(|| anyhow::anyhow!("No filesystem found."))?
                .calculate_sum_of_directories_sizes_where_each_size_max(100_000);

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
        assert_eq!(&derived_filesystem.to_string(), TEST_VISUAL_REPRESENTATION);

        Ok(())
    }

    #[test]
    fn test_part_2_default() -> anyhow::Result<()> {
        // Arrange
        let derived_filesystem = CommandHistory::from_str(TEST_INPUT)?
            .derive_filesystem()?
            .ok_or_else(|| anyhow::anyhow!("No filesystem found."))?;

        // Act
        let total_size_of_deletable_directory = derived_filesystem
            .find_directory_size_to_delete_to_free_enough_space(70_000_000, 30_000_000)?;

        // Assert
        assert_eq!(total_size_of_deletable_directory, 24_933_642);

        Ok(())
    }
}
