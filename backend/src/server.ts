import Fastify from "fastify";
import cors from "@fastify/cors";
import duckdb from "duckdb";
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Initialize Fastify
const fastify = Fastify({ logger: true });

// Register CORS
await fastify.register(cors, {
  origin: "*",
});

// DuckDB Database configuration
// Using a local file database in Docker, falls back to memory for testing
const dataDir = path.join(process.cwd(), "data");
if (!fs.existsSync(dataDir)) {
  fs.mkdirSync(dataDir, { recursive: true });
}
const dbPath = path.join(dataDir, "cab_logger.db");
const db = new duckdb.Database(dbPath);
const conn = db.connect();

// Utility function to execute a query and return a promise
function runQuery(sql: string, params: any[] = []): Promise<any[]> {
  return new Promise((resolve, reject) => {
    const callback = (err: any, rows: any) => {
      if (err) {
        reject(err);
      } else {
        resolve(rows || []);
      }
    };

    if (params && params.length > 0) {
      conn.all(sql, ...params, callback);
    } else {
      conn.all(sql, callback);
    }
  });
}

// Initialize tables and seed database
async function initDatabase() {
  try {
    // Create Bookings Table
    await runQuery(`
      CREATE TABLE IF NOT EXISTS bookings (
        id VARCHAR PRIMARY KEY,
        employee_name VARCHAR,
        department VARCHAR,
        pickup_location VARCHAR,
        drop_location VARCHAR,
        date_time VARCHAR,
        vendor_name VARCHAR,
        vendor_email VARCHAR,
        status VARCHAR,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
      )
    `);

    // Create Sent Emails Table
    await runQuery(`
      CREATE TABLE IF NOT EXISTS sent_emails (
        id VARCHAR PRIMARY KEY,
        booking_id VARCHAR,
        to_email VARCHAR,
        subject VARCHAR,
        body TEXT,
        sent_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
      )
    `);

    // Seed data if empty
    const countResult = await runQuery("SELECT COUNT(*) as count FROM bookings");
    const count = Number(countResult[0]?.count ?? 0);

    if (count === 0) {
      console.log("[DB] No bookings found. Seeding initial data into DuckDB...");

      const seedBookings = [
        {
          id: "#0001",
          employee_name: "Marcus Vance",
          department: "Operations",
          pickup_location: "HQ Block A",
          drop_location: "SFO Terminal 2",
          date_time: new Date(Date.now() - 1 * 60 * 60 * 1000).toISOString().slice(0, 16),
          vendor_name: "Metro Cab Link",
          vendor_email: "dispatch@metrocablink.com",
          status: "Sent to Vendor",
        },
        {
          id: "#0002",
          employee_name: "Elena Rostova",
          department: "Engineering",
          pickup_location: "West Labs",
          drop_location: "Metro Station Plaza",
          date_time: new Date(Date.now() - 3 * 60 * 60 * 1000).toISOString().slice(0, 16),
          vendor_name: "Swift Fleet Logistics",
          vendor_email: "bookings@swiftfleet.net",
          status: "Sent to Vendor",
        },
        {
          id: "#0003",
          employee_name: "Jameson Patel",
          department: "Legal & Finance",
          pickup_location: "HQ Block B",
          drop_location: "Downtown Courthouse",
          date_time: new Date(Date.now() - 5 * 60 * 60 * 1000).toISOString().slice(0, 16),
          vendor_name: "Horizon Shuttles",
          vendor_email: "dispatch@horizonshuttles.co",
          status: "Pending",
        },
        {
          id: "#0004",
          employee_name: "Sarah Jenkins",
          department: "HR & Recruitment",
          pickup_location: "Central Annex",
          drop_location: "Hotel Grand Palace",
          date_time: new Date(Date.now() - 25 * 60 * 60 * 1000).toISOString().slice(0, 16),
          vendor_name: "Beacon Town Cars",
          vendor_email: "reservation@beacontowncars.com",
          status: "Sent to Vendor",
        },
      ];

      for (const b of seedBookings) {
        await runQuery(
          `INSERT INTO bookings (id, employee_name, department, pickup_location, drop_location, date_time, vendor_name, vendor_email, status) 
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)`,
          [
            b.id,
            b.employee_name,
            b.department,
            b.pickup_location,
            b.drop_location,
            b.date_time,
            b.vendor_name,
            b.vendor_email,
            b.status,
          ]
        );

        if (b.status === "Sent to Vendor") {
          const emailId = `EM-${Math.random().toString(36).substr(2, 9).toUpperCase()}`;
          const emailBody = `Hello ${b.vendor_name} Team,

This is an automated booking request from Cab Logger. Please dispatch a cab with the following trip details:

- Booking Reference: ${b.id}
- Passenger Name: ${b.employee_name} (${b.department})
- Pickup Location: ${b.pickup_location}
- Destination: ${b.drop_location}
- Schedule Date/Time: ${b.date_time.replace("T", " ")}

Please confirm receipt of this dispatch.

Regards,
Cab Logger Dispatch Desk`;

          await runQuery(
            `INSERT INTO sent_emails (id, booking_id, to_email, subject, body) VALUES (?, ?, ?, ?, ?)`,
            [emailId, b.id, b.vendor_email, `URGENT CAB DISPATCH REQUEST: Ticket ${b.id}`, emailBody]
          );
        }
      }
      console.log("[DB] DuckDB seed injection complete.");
    }
  } catch (err) {
    console.error("[DB] Error initializing DuckDB:", err);
  }
}

// Initialize tables
await initDatabase();

// --- Fastify Endpoints ---

// 1. Get all bookings sorted newest first
fastify.get("/api/bookings", async (request, reply) => {
  try {
    const rows = await runQuery(`SELECT * FROM bookings ORDER BY created_at DESC`);
    // Map camelCase to snake_case compatibility for Rust frontend
    const mapped = rows.map((r) => ({
      id: r.id,
      employeeName: r.employee_name,
      department: r.department,
      pickupLocation: r.pickup_location,
      dropLocation: r.drop_location,
      dateTime: r.date_time,
      vendorName: r.vendor_name,
      vendorEmail: r.vendor_email,
      status: r.status,
      createdAt: r.created_at,
    }));
    return mapped;
  } catch (err: any) {
    reply.status(500).send({ error: err.message });
  }
});

// 2. Count bookings logged today (for odometer ticker)
fastify.get("/api/bookings/today-count", async (request, reply) => {
  try {
    const todayStr = new Date().toISOString().slice(0, 10); // "YYYY-MM-DD"
    // Search entries created on the current calendar date
    const rows = await runQuery(
      `SELECT COUNT(*) as count FROM bookings WHERE STRFTIME(created_at, '%Y-%m-%d') = ?`,
      [todayStr]
    );
    const count = Number(rows[0]?.count ?? 0);
    return { count };
  } catch (err: any) {
    reply.status(500).send({ error: err.message });
  }
});

// 3. Get all mock dispatched emails
fastify.get("/api/sent-emails", async (request, reply) => {
  try {
    const rows = await runQuery(`SELECT * FROM sent_emails ORDER BY sent_at DESC`);
    const mapped = rows.map((r) => ({
      id: r.id,
      bookingId: r.booking_id,
      to: r.to_email,
      subject: r.subject,
      body: r.body,
      sentAt: r.sent_at,
    }));
    return mapped;
  } catch (err: any) {
    reply.status(500).send({ error: err.message });
  }
});

// 4. Create a new dispatch ticket and immediately notify vendor (mock email)
interface CreateBookingBody {
  employeeName: string;
  department: string;
  pickupLocation: string;
  dropLocation: string;
  dateTime: string;
  vendorName: string;
  vendorEmail: string;
}

fastify.post<{ Body: CreateBookingBody }>("/api/bookings", async (request, reply) => {
  try {
    const {
      employeeName,
      department,
      pickupLocation,
      dropLocation,
      dateTime,
      vendorName,
      vendorEmail,
    } = request.body;

    if (!employeeName || !department || !pickupLocation || !dropLocation || !dateTime || !vendorName || !vendorEmail) {
      return reply.status(400).send({ error: "All fields are required" });
    }

    // Determine next sequential ID
    const rows = await runQuery("SELECT id FROM bookings");
    const nextNum = rows.length + 1;
    const bId = `#${String(nextNum).padStart(4, "0")}`;

    // Store in DuckDB
    await runQuery(
      `INSERT INTO bookings (id, employee_name, department, pickup_location, drop_location, date_time, vendor_name, vendor_email, status)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)`,
      [bId, employeeName, department, pickupLocation, dropLocation, dateTime, vendorName, vendorEmail, "Sent to Vendor"]
    );

    // Mock Email content
    const emailId = `EM-${Math.random().toString(36).substr(2, 9).toUpperCase()}`;
    const formattedTime = dateTime.replace("T", " ");
    const emailBody = `Hello ${vendorName} Team,

This is an automated booking request from Cab Logger. Please dispatch a cab with the following trip details:

- Booking Reference: ${bId}
- Passenger Name: ${employeeName} (${department})
- Pickup Location: ${pickupLocation}
- Destination: ${dropLocation}
- Schedule Date/Time: ${formattedTime}

Please confirm receipt of this dispatch.

Regards,
Cab Logger Dispatch Desk`;

    await runQuery(
      `INSERT INTO sent_emails (id, booking_id, to_email, subject, body) VALUES (?, ?, ?, ?, ?)`,
      [emailId, bId, vendorEmail, `URGENT CAB DISPATCH REQUEST: Ticket ${bId}`, emailBody]
    );

    console.log(`[DUCKDB BACKEND] Dispatched email created for booking ${bId}`);

    const newBooking = {
      id: bId,
      employeeName,
      department,
      pickupLocation,
      dropLocation,
      dateTime,
      vendorName,
      vendorEmail,
      status: "Sent to Vendor",
    };

    return reply.status(201).send({ booking: newBooking, email: { id: emailId, to: vendorEmail, body: emailBody } });
  } catch (err: any) {
    reply.status(500).send({ error: err.message });
  }
});

// 5. Send/update a pending dispatch ticket to Sent status
fastify.post<{ Params: { id: string } }>("/api/bookings/:id/send", async (request, reply) => {
  try {
    const { id } = request.params;
    const rows = await runQuery(`SELECT * FROM bookings WHERE id = ?`, [id]);
    const booking = rows[0];

    if (!booking) {
      return reply.status(404).send({ error: "Booking not found" });
    }

    if (booking.status === "Sent to Vendor") {
      return reply.status(400).send({ error: "Booking already sent to vendor" });
    }

    // Update status
    await runQuery(`UPDATE bookings SET status = 'Sent to Vendor' WHERE id = ?`, [id]);

    // Create corresponding sent email
    const emailId = `EM-${Math.random().toString(36).substr(2, 9).toUpperCase()}`;
    const formattedTime = booking.date_time.replace("T", " ");
    const emailBody = `Hello ${booking.vendor_name} Team,

This is an automated booking request from Cab Logger. Please dispatch a cab with the following trip details:

- Booking Reference: ${booking.id}
- Passenger Name: ${booking.employee_name} (${booking.department})
- Pickup Location: ${booking.pickup_location}
- Destination: ${booking.drop_location}
- Schedule Date/Time: ${formattedTime}

Please confirm receipt of this dispatch.

Regards,
Cab Logger Dispatch Desk`;

    await runQuery(
      `INSERT INTO sent_emails (id, booking_id, to_email, subject, body) VALUES (?, ?, ?, ?, ?)`,
      [emailId, booking.id, booking.vendor_email, `URGENT CAB DISPATCH REQUEST: Ticket ${booking.id}`, emailBody]
    );

    return { success: true };
  } catch (err: any) {
    reply.status(500).send({ error: err.message });
  }
});

// Start server
const start = async () => {
  try {
    const port = Number(process.env.PORT) || 8095;
    await fastify.listen({ port, host: "0.0.0.0" });
    console.log(`[BACKEND] Cab Logger server starting on port ${port}...`);
  } catch (err) {
    fastify.log.error(err);
    process.exit(1);
  }
};

start();
