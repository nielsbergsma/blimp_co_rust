import { Miniflare,Log, LogLevel } from "miniflare";

export function serve() {
    return new Miniflare({
        log: new Log(LogLevel.DEBUG),
        port: 5000,
        workers: [
            {
                name: "scheduling_api",
                modules: true,
                modulesRules: [
                    {type: "CompiledWasm", include: ["build/scheduling_api/*.wasm"]},
                ],
                scriptPath: "build/scheduling_api/shim.mjs",
                routes: ["http://127.0.0.1/flights", "http://127.0.0.1/airships", "http://127.0.0.1/airfields"],
                bindings: {
                    api_key: "1234",
                },
                queueProducers: {
                    reservation_queue: "reservation_rs_queue",
                    scheduling_queue: "scheduling_rs_queue"
                },
                durableObjects: {
                    scheduling_objects: "SchedulingRepository"
                },
                r2Buckets: ["scheduling_rs_bucket"],
                queueConsumers: {
                    "scheduling_rs_queue": {
                        maxBatchSize: 5,
                        maxBatchTimeout: 1,
                        maxRetries: 1,
                        deadLetterQueue: "scheduling_rs_queue-dlq"
                    }
                },
                compatibilityFlags: ["nodejs_compat"],
                compatibilityDate: "2024-09-23"
            },
            {
                name: "backoffice_site",
                routes: ["http://127.0.0.1/flight-scheduling", "http://127.0.0.1/reservations", "http://127.0.0.1/js/*", "http://127.0.0.1/img/*", "http://127.0.0.1/css/*"],
                modules: true,
                script: `
                    export default {
                        async fetch(request, env, ctx) {
                            const url = new URL(request.url);
                            return await fetch("http://localhost:5001" + url.pathname);
                        }
                    }
                `,
                r2Buckets: ["scheduling_bucket"],
            },
            {
                name: "reservation_site",
                routes: ["http://127.0.0.1/journey-around-the-north-atlantic", "http://127.0.0.1/journey-around-the-north-atlantic/*"],
                modules: true,
                script: `
                    export default {
                        async fetch(request, env, ctx) {
                            const url = new URL(request.url);
                            return await fetch("http://localhost:5003" + url.pathname);
                        }
                    }
                `,
                r2Buckets: ["scheduling_rs_bucket"],
            },
            {
                name: "reservation_api",
                modules: true,
                modulesRules: [
                    {type: "CompiledWasm", include: ["build/reservation_api/*.wasm"]},
                ],
                scriptPath: "build/reservation_api/shim.mjs",
                routes: ["http://127.0.0.1/journeys", "http://127.0.0.1/reservations", "http://127.0.0.1/reservations/*"],
                bindings: {
                    api_key: "1234",
                },
                queueProducers: {
                    reservation_queue: "reservation_rs_queue",
                },
                durableObjects: {
                    reservation_objects: "ReservationRepository"
                },
                r2Buckets: ["reservation_rs_bucket"],
                queueConsumers: {
                    "reservation_rs_queue": {
                        maxBatchSize: 5,
                        maxBatchTimeout: 1,
                        maxRetries: 1,
                        deadLetterQueue: "reservation_rs_queue-dlq"
                    }
                },
                compatibilityFlags: ["nodejs_compat"],
                compatibilityDate: "2024-09-23"
            },
            {
                name: "buckets",
                routes: ["http://127.0.0.1/buckets/reservation/journeys", "http://127.0.0.1/buckets/scheduling/dashboard", "http://127.0.0.1/buckets/reservation/availability/*"],
                modules: true,
                script: `
                  export default {
                    async fetch(request, env, ctx) {
                      const url = new URL(request.url);
                      const path = url.pathname.split("/");
                      const bucket = path[2] + "_rs_bucket";
                      const resource = path.slice(3).join("/");
                      
                      const object = await env[bucket].get(resource);
                      if (object) {
                        const value = await object.json();
                        return Response.json(value);
                      }
                      else {
                        return Response.json({ error: "not found"}, { status: 404 })
                      }
                    }
                  }
                `,
                r2Buckets: ["reservation_rs_bucket", "scheduling_rs_bucket"],
            },
        ]
    });
}

export async function teardown(instance) {
    await instance.dispose();
}

serve();