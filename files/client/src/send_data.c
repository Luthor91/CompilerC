#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <netinet/in.h>

typedef struct {
    int data1;
    char data2[100];
} DatabaseRecord;

int main() {
    // Créez une socket pour la connexion au serveur SGDB
    int clientSocket = socket(AF_INET, SOCK_STREAM, 0);
    if (clientSocket < 0) {
        perror("Erreur lors de la création de la socket du client");
        exit(1);
    }

    struct sockaddr_in serverAddr;
    memset(&serverAddr, 0, sizeof(serverAddr));
    serverAddr.sin_family = AF_INET;
    serverAddr.sin_port = htons(12345); // Port du serveur SGDB
    serverAddr.sin_addr.s_addr = inet_addr("IP_DU_SERVEUR"); // Remplacez par l'IP du serveur

    // Établir la connexion au serveur SGDB
    if (connect(clientSocket, (struct sockaddr *)&serverAddr, sizeof(serverAddr)) < 0) {
        perror("Erreur lors de la connexion au serveur");
        exit(1);
    }

    // Exemple de données à envoyer au serveur SGDB
    DatabaseRecord record;
    record.data1 = 42;
    strncpy(record.data2, "Données de test", sizeof(record.data2));

    // Envoyer les données au serveur SGDB
    if (send(clientSocket, &record, sizeof(record), 0) < 0) {
        perror("Erreur lors de l'envoi des données au serveur");
        exit(1);
    }

    // Fermer la connexion client
    close(clientSocket);

    return 0;
}
