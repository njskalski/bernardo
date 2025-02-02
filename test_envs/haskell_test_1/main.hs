-- main.hs
module Main where

import System.Environment (getArgs)

-- Function to generate a greeting message
greet :: String -> String
greet name = "Hello, " ++ name ++ "!"

-- Main function
main :: IO ()
main = do
    args <- getArgs
    let name = if null args then "World" else head args
    putStrLn (greet name)
