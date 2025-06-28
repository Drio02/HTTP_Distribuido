# Comando para levantar el los contendores de todo el sistema
docker-compose up --build

# Comando para ver el estado de los workers
curl http://localhost:8080/workers | jq .

# Machote de como ejecutar comandos
curl http://localhost:8080/{endpoint que desea}


# Comandos para probar la eficiencia del calculo de PI
time curl http://localhost:8080/montecarlo?points=20000000

time curl http://localhost:8080/montecarlo?points=2000000000

time curl http://localhost:8080/montecarlo?points=20000000000
