# CompilerC


Application servant à compiler un projet en C sur Windows, à tester sur Linux / Ubuntu

## Fonctionnement 

On assume que l'exécutable main.exe est placé à la racine du répertoire du projet C.
On assume que le nom du répertoire parent est le nom du projet C.

Une fois lancé, l'exécutable va créer un dossier du même nom que le dossier parent.
Dans ce dossier seront placé les fichiers C, Header, DLL, Output.

La première commande à être exécuté par l'application sera pour build les fichiers sources en fichiers .o .
La deuxième commande à être exécuté par l'application sera pour build l'exécutable.
Ensuite l'exécutable sera lancé.

## Pré-requis

compilateur GCC.

## Installation

Il y a juste besoin de placer l'exécutable à la racine du projet à compiler.

## Améliorations 

- Système de gestion de version du projet
- Possibilité de l'exécuter n'importe où dans le pc du moment que le projet à compiler est renseigné
- Donner la possibilité de l'exécuter ou pas après la compilation
- Installation de GCC si non présent.
