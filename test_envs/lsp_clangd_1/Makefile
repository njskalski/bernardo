# Compiler and flags
CXX = clang++
CXXFLAGS = -Wall -Wextra -pedantic -std=c++11

# Directories
SRC_DIR = ./src
OBJ_DIR = ./obj
BIN_DIR = ./bin

# Source files
MAIN_SRC = $(SRC_DIR)/main.cpp
HELLO_SRC = $(SRC_DIR)/hello.cpp

# Object files
MAIN_OBJ = $(OBJ_DIR)/main.o
HELLO_OBJ = $(OBJ_DIR)/hello.o

# Output binary
EXECUTABLE = $(BIN_DIR)/myapp

# Default target (all)
all: $(EXECUTABLE)

# Linking object files to generate binary
$(EXECUTABLE): $(MAIN_OBJ) $(HELLO_OBJ)
	@mkdir -p $(BIN_DIR)
	$(CXX) $(CXXFLAGS) $^ -o $@

# Compiling main source file
$(MAIN_OBJ): $(MAIN_SRC)
	@mkdir -p $(OBJ_DIR)
	$(CXX) $(CXXFLAGS) -c $< -o $@

# Compiling hello source file
$(HELLO_OBJ): $(HELLO_SRC) $(SRC_DIR)/hello.h
	@mkdir -p $(OBJ_DIR)
	$(CXX) $(CXXFLAGS) -c $< -o $@

# Clean intermediate object files and the binary
clean:
	$(RM) -r $(OBJ_DIR) $(BIN_DIR)

# Clean and rebuild
rebuild: clean all

# Phony targets
.PHONY: all clean rebuild
