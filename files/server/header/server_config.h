#ifndef SERVER_CONFIG_H
#define SERVER_CONFIG_H

#define SERVER_PORT 8080
#define MAX_CONNECTIONS 5

struct server_config {
    int port;
    int max_connections;
    // Ajoutez d'autres paramètres de configuration si nécessaire
};

extern struct server_config server_config;

#endif
