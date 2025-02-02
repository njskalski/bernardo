#!/bin/bash
#generated with chatgpt

# Function to perform a greeting
greet_user() {
    echo "Hello, $1! Welcome to the Bash script."
}

# Function to say goodbye
say_goodbye() {
    echo "Goodbye, $1! Have a wonderful day!"
}

# Main function
main() {
    # Prompt the user for their name
    echo -n "Enter your name: "
    read user_name

    # Call the greet_user function with the user's name
    greet_user "$user_name"

    # Call the say_goodbye function with the user's name
    say_goodbye "$user_name"
}

# Execute the main function
main
