#include "../header/server_config.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifdef _WIN32
typedef int socklen_t; // Définition de socklen_t pour Windows
#include <winsock2.h>
#include <windows.h>
#else
#include <sys/socket.h>
#include <netinet/in.h>
#include <unistd.h>
#endif

int main() {
    int server_socket;
    struct sockaddr_in server_addr;
    
    // Ces variables doivent être déclarées ici pour être accessibles dans les deux cas
    int client_socket;
    struct sockaddr_in client_addr;
    socklen_t client_addr_len;

    // Création de la socket serveur en utilisant les paramètres de server_config
    server_socket = socket(AF_INET, SOCK_STREAM, 0);
    if (server_socket < 0) {
        perror("Erreur lors de la création du socket");
        exit(1);
    }

    server_addr.sin_family = AF_INET;
    server_addr.sin_addr.s_addr = INADDR_ANY;
    server_addr.sin_port = htons(server_config.port);

    if (bind(server_socket, (struct sockaddr*)&server_addr, sizeof(server_addr)) < 0) {
        perror("Erreur lors de la liaison du socket");
        exit(1);
    }

    // Mise en écoute des connexions
    listen(server_socket, server_config.max_connections);

    while (1) {
#ifdef _WIN32
        client_socket = accept(server_socket, (struct sockaddr*)&client_addr, &client_addr_len);
#else
        client_addr_len = sizeof(client_addr);
        client_socket = accept(server_socket, (struct sockaddr*)&client_addr, &client_addr_len);
#endif

        if (client_socket < 0) {
            perror("Erreur lors de l'acceptation de la connexion client");
            continue; // Gérer l'erreur et continuer la boucle
        }

        // Gérer la connexion client dans un thread ou un processus séparé
        // Ici, vous pouvez gérer la réception et le traitement des requêtes

#ifdef _WIN32
        closesocket(client_socket); // Fermer la socket du client sous Windows
#else
        close(client_socket); // Fermer la socket du client sous Linux
#endif
    }

#ifdef _WIN32
    closesocket(server_socket); // Fermer la socket serveur sous Windows
    WSACleanup(); // Libérer les ressources Winsock
#else
    close(server_socket); // Fermer la socket serveur sous Linux
#endif

    return 0;
}
