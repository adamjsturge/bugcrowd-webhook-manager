# Bugcrowd Webhook Manager
Basically an easy way to split up webhook requests from bugcrowd into more webhooks

## Setup
Copy the .env.copy to .env and fill in the values
To generate a hash we recommend `openssl rand -base64 32` this will generate a 32 character hash you can increase number of characters if you want

## To Run Locally
To run locally just use `cargo run` this will expose the port at http://127.0.0.1:3000/

## To Run on a Server
Use `docker compose up -d` this will start your server to be exposed on port 80 and 443
For this to work you will need to have the domain pointing to the server
All certs are handled by traefik

## License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details

## Misc
This project was to learn more about rust and how to use it; while, improving the current workflow for bugcrowd. If you have any suggestions or want to contribute feel free to open a PR or issue
Hopefully this can be useful to someone else
