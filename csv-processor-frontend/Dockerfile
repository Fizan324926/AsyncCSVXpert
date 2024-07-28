FROM node:20

# Set the working directory inside the container
WORKDIR /app

# Copy the package.json and yarn.lock or package-lock.json
COPY package.json package-lock.json ./

# Install dependencies
RUN npm install

# Copy the rest of the frontend code
COPY . .

# Expose the port the app runs on
EXPOSE 3000

# Run the React app
ENTRYPOINT [ "npm", "start" ]
