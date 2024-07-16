# AsyncCSVXpert

AsyncCSVXpert is a web application that allows users to upload CSV files, processes the URLs contained within, performs HTTP HEAD requests, and returns a CSV with detailed response information. The project consists of a Rust backend and a React frontend.

## Table of Contents
- [Prerequisites](#prerequisites)
- [Setup](#setup)
- [Building the Backend](#building-the-backend)
- [Running the Backend](#running-the-backend)
- [Building the Frontend](#building-the-frontend)
- [Running the Frontend](#running-the-frontend)
- [Usage](#usage)


## Prerequisites

Before you begin, ensure you have the following installed:
- Rust (including cargo)
- Node.js and npm (for the React frontend)

## Setup

Clone the repository:
```sh
cd AsyncCSVXpert
```
Navigate to the Rust backend directory and install dependencies:

```sh
cd csv_processor
cargo build
```
Navigate to the React frontend directory and install dependencies:

```sh
cd csv-processor-frontend
npm install
```
## Building the Backend
To build the Rust backend, navigate to the csv_processor directory and run:

```sh
cargo build --release
```
The compiled binary will be located in the target/release directory.

## Running the Backend
To run the Rust backend, execute:

```sh
cargo run
```
The backend server will start, typically on http://localhost:8080.

## Building the Frontend
To build the React frontend, navigate to the csv-processor-frontend directory and run:

```sh
npm run build
```
The build output will be located in the build directory.

## Running the Frontend
To run the React frontend in development mode, execute:

```sh
npm start
```
The frontend will start, typically on http://localhost:3000.

## Usage
- Open your web browser and navigate to http://localhost:3000.
- Upload a CSV file containing id and URL columns.
- The application will process the URLs, perform HTTP HEAD requests, and display the results.
- Download the processed CSV file with the results.

