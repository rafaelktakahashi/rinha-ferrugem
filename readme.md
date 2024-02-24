# ferrugem (Rinha de backend do zanfranceschi, implementado em Rust)

## Para rodar o projeto

```bash
docker build -t ferrugem_rust -f Dockerfile.rust .
docker-compose up -d
```

Comandos podem ser diferentes dependendo da plataforma. Eu preciso rodar o compose usando `sudo docker compose up` no meu WSL2.

## Detalhes da stack

Aplicação Rust, com servidor http implementado com actix-web. Banco de dados postgres 16, com client sqlx. O load balancer é um nginx somente com um arquivo de configuração simples.

## Sobre o código

O código é ruim. É eficiente, mas é ruim. Ele prioriza performance de uma maneira desproporcionada e deixa de lado tudo o que importa na vida. Mas funciona.