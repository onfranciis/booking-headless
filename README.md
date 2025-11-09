# Booking Headless Backend

Booking Headless is an open-source backend for service providers. It enables businesses and professionals to connect with their customers, manage appointments, and integrate bookings with Google Calendar.

**Status:** 🚧 _Active development_ — features and APIs may change frequently. Feedback and contributions are highly encouraged!

## Features

- User registration and authentication
- Service creation and management
- Appointment booking and scheduling
- Google Calendar integration for automatic event storage
- RESTful API endpoints
- Built with Rust and Actix Web
- PostgreSQL database support

## Getting Started

### Prerequisites

- Rust (latest stable)
- PostgreSQL
- Google API credentials (for calendar integration)

### Installation

1. Clone the repository:
   ```sh
   git clone https://github.com/onfranciis/booking-headless.git
   cd booking-headless
   ```
2. Set up your PostgreSQL database and update the connection string in `.env`.
3. Run database migrations:
   ```sh
   cargo install sqlx-cli
   sqlx migrate run
   ```
4. Build and run the server:
   ```sh
   cargo run
   ```

## Usage

Interact with the API using tools like [Postman](https://www.postman.com/) or [curl](https://curl.se/). See the `/users`, `/services`, and `/appointments` endpoints for user, service, and booking management.

### Example: Book an Appointment

```sh
curl -X POST http://localhost:8080/appointments \
  -H "Content-Type: application/json" \
  -d '{"service_id": "...", "customer_name": "...", "datetime": "..."}'
```

## Google Calendar Integration

When a customer books an appointment, the backend automatically creates a corresponding event in the service provider's Google Calendar (requires Google API setup).

## Development

This project is in active development. To contribute:

1. Fork the repository and create a feature branch.
2. Make your changes and add tests if possible.
3. Open a pull request describing your changes.

### Running Tests

```sh
cargo test
```

### Code Style

Format code with:

```sh
cargo fmt
```

## Support & Feedback

If you encounter issues or have feature requests, please open an issue on GitHub. For questions, reach out via Discussions or email the maintainer.

## License

This project is licensed under the MIT License.
