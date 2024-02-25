# ferrugem (Rinha de backend do zanfranceschi, implementado em Rust)

Rust

The roil of stars, as iron oxidates  
Foretells the slow decay of previous days  
To walk across its wicked vacant gates  
Does coldly all our yesterdays rephrase

Oh time, thy pyramids  
Against the lowest realms cannot compare  
'Tis made of what our faculties forbid  
Patterns on sand, far eons writ on air

Bane of the vain, traverse the ageless rains  
That pour on wheels, corrode both mind and gear  
But he who spies what after dust remains  
Will see his shackles yield, and disappear

Of deathlessness a sign supreme  
Oxide, a liberty ultime

## Para rodar o projeto

```bash
docker build -t ferrugem_rust -f Dockerfile.rust .
docker-compose up -d
```

Comandos podem ser diferentes dependendo da plataforma. Eu preciso rodar o compose usando `sudo docker compose up` no meu WSL2.

## Detalhes da stack

Aplicação Rust, com servidor http implementado com actix-web. Banco de dados postgres 16, com client sqlx. O load balancer é um nginx somente com um arquivo de configuração simples.

## Sobre o código

O código é eficiente, mas não deve ser modelo para ninguém. Ele prioriza performance de uma maneira desproporcionada e deixa de lado tudo o que importa na vida. Mas funciona. Compartilho aqui o que é possível de se fazer, mas código real em produção não teria essa aparência.
