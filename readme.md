# ferrugem (Rinha de backend do zanfranceschi, implementado em Rust)

## Para rodar o projeto

```bash
docker build -t ferrugem_rust .
docker-compose up -d
```

## Detalhes da stack

Servidor http implementado com actix-web. Banco de dados postgres, com client sqlx.