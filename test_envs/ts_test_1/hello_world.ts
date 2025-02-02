// hello.ts

function sayHello(): void {
    console.log("Hello, World!");
}

function greetUser(name: string): void {
    console.log(`Hello, ${name}!`);
}

function main(): void {
    const args = process.argv.slice(2);

    if (args.length > 0) {
        greetUser(args[0]);
    } else {
        sayHello();
    }
}

main();
