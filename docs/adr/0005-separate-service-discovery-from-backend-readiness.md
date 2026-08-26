# Separate Service discovery from backend readiness

Once the Host registry is published, exact-version discovery returns every Service compiled into that Host even when its native backend is not ready. Service status and operations report runtime readiness. The new Rust SDK does not fall back to the legacy host export. During migration, a header-prefixed Legacy SA-MP Service wrapper may point to the old API table; later it is replaced or removed like any other pre-1.0 Service.
