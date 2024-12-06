import re
from collections import defaultdict
import subprocess
from langdetect import detect
import argparse
import os

def get_rust_comments(directory):
    """
    Gets all comments using ripgrep and detects their language.
    
    Args:
        directory (str): The relative path to search for Rust files
        
    Returns:
        dict: Comments organized by detected language
    """
    # Ensure the directory exists
    if not os.path.exists(directory):
        raise FileNotFoundError(f"Directory not found: {directory}")

    # Execute ripgrep to find lines with comments, including filename and line number
    result = subprocess.run(
        ["rg", "//", "--line-number", "--with-filename"],
        capture_output=True,
        text=True,
        cwd=directory
    )

    comments = result.stdout.split('\n')

    # Dictionary to store comments by language
    comments_by_lang = defaultdict(list)

    for line in comments:
        if not line.strip():
            continue

        # Parse ripgrep output (format: file:line:content)
        parts = line.split(':', 2)
        if len(parts) < 3:
            continue

        filename, line_num, content = parts

        # Extract only the comment text (after //)
        comment_match = re.search(r'//\s*(.*)', content)
        if comment_match:
            comment_text = comment_match.group(1).strip()
            if len(comment_text) > 3:  # Ignore very short comments
                try:
                    # Detect comment language
                    lang = detect(comment_text)
                    # Store complete information
                    comments_by_lang[lang].append({
                        'file': filename,
                        'line': line_num,
                        'content': content.strip(),
                        'comment_text': comment_text
                    })
                except:
                    comments_by_lang['unknown'].append({
                        'file': filename,
                        'line': line_num,
                        'content': content.strip(),
                        'comment_text': comment_text
                    })

    return comments_by_lang

def print_comments_by_language(comments_by_lang):
    """
    Prints comments organized by language, showing their location.
    
    Args:
        comments_by_lang (dict): Dictionary containing comments organized by language
    """
    print("\n=== Spanish Comments (es) ===")
    for comment in comments_by_lang.get('es', []):
        print(f"\nFile: {comment['file']}")
        print(f"Line: {comment['line']}")
        print(f"Content: {comment['content']}")

    # print("\n=== English Comments (en) ===")
    # for comment in comments_by_lang.get('en', []):
    #     print(f"\nFile: {comment['file']}")
    #     print(f"Line: {comment['line']}")
    #     print(f"Content: {comment['content']}")

    # if comments_by_lang.get('unknown', []):
    #     print("\n=== Unidentified ===")
    #     for comment in comments_by_lang['unknown']:
    #         print(f"\nFile: {comment['file']}")
    #         print(f"Line: {comment['line']}")
    #         print(f"Content: {comment['content']}")

def generate_translation_todo(comments_by_lang):
    """
    Generates a summary of files that need translation.
    
    Args:
        comments_by_lang (dict): Dictionary containing comments organized by language
    """
    print("\n=== Files to Translate Summary ===")
    files_to_translate = defaultdict(int)

    for comment in comments_by_lang.get('es', []):
        files_to_translate[comment['file']] += 1

    for file, count in files_to_translate.items():
        print(f"\n{file}: {count} Spanish comments")

def main():
    """Main function to handle command line arguments and execute the script."""
    parser = argparse.ArgumentParser(
        description='Detect language of Rust code comments in a specified directory'
    )
    parser.add_argument(
        'directory',
        nargs='?',
        default='.',
        help='Relative path to the directory containing Rust files (default: current directory)'
    )

    args = parser.parse_args()

    try:
        comments = get_rust_comments(args.directory)
        print_comments_by_language(comments)
        generate_translation_todo(comments)
    except FileNotFoundError as e:
        print(f"Error: {e}")
    except subprocess.CalledProcessError:
        print(f"Error: Unable to search in directory: {args.directory}")
    except Exception as e:
        print(f"Error: An unexpected error occurred: {e}")

if __name__ == "__main__":
    main()