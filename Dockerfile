# Etapa de build
FROM rust:1.81 as builder

WORKDIR /app

# Copiamos el Cargo.toml 
COPY Cargo.toml .


COPY src ./src

# Ahora copiamos el resto del código
COPY . .

# Compilamos en release
RUN cargo build --release

# Etapa final
FROM debian:bookworm

# Creamos una carpeta para el binario y la DB
RUN mkdir /app
WORKDIR /app

# Copiamos el binario desde la etapa de builder
COPY --from=builder /app/target/release/series-tracker /usr/local/bin/series-tracker

# Puertos expuestos
EXPOSE 8080

# Comando final
CMD ["series-tracker"]