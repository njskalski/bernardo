class Greeter:
    def __init__(self, message):
        self.message = message

    def greet(self):
        print(self.message)

# Create an instance of the Greeter class with the message "Hello, World!"
greeter = Greeter("Hello, World!")

# Call the greet method to print the message
greeter.greet()
