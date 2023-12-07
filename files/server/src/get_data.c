#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <unistd.h>

// Structure pour stocker les données de la base de données (c'est un exemple simple)
typedef struct {
    int data1;
    char data2[100];
} DatabaseRecord;

int main() {
    // Créez une socket pour écouter les connexions entrantes
    int serverSocket = socket(AF_INET, SOCK_STREAM, 0);
    if (serverSocket < 0) {
        perror("Erreur lors de la création de la socket");
        exit(1);
    }

    struct sockaddr_in serverAddr;
    memset(&serverAddr, 0, sizeof(serverAddr);
    serverAddr.sin_family = AF_INET;
    serverAddr.sin_port = htons(12345); // Port d'écoute de votre choix
    serverAddr.sin_addr.s_addr = INADDR_ANY;

    if (bind(serverSocket, (struct sockaddr *)&serverAddr, sizeof(serverAddr)) < 0) {
        perror("Erreur lors de la liaison de la socket");
        exit(1);
    }

    listen(serverSocket, 5);

    printf("En attente de connexions...\n");

    while (1) {
        struct sockaddr_in clientAddr;
        socklen_t clientAddrLen = sizeof(clientAddr);

        // Acceptez une connexion entrante
        int clientSocket = accept(serverSocket, (struct sockaddr *)&clientAddr, &clientAddrLen);
        if (clientSocket < 0) {
            perror("Erreur lors de l'acceptation de la connexion");
            exit(1);
        }

        printf("Nouvelle connexion acceptée\n");

        // Exemple de traitement : recevoir des données du client
        DatabaseRecord record;
        if (recv(clientSocket, &record, sizeof(record), 0) < 0) {
            perror("Erreur lors de la réception des données du client");
            close(clientSocket);
            continue;
        }

        // Exemple de traitement : afficher les données reçues
        printf("Données reçues : data1=%d, data2=%s\n", record.data1, record.data2);

        // Vous pouvez maintenant traiter ces données dans votre base de données personnalisée

        // Fermer la connexion client
        close(clientSocket);
    }

    // Fermez la socket du serveur (ce code ne sera pas atteint dans cette boucle infinie)
    close(serverSocket);

    return 0;
}
