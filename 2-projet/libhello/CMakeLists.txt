# Nous voulons un cmake "récent" pour utiliser les dernières fonctionnalités
cmake_minimum_required(VERSION 3.0)

# Notre projet est étiqueté libhello
project(libhello)

# Créé des variables avec les fichiers à compiler
set(SRCS
	hello.c
	)
	
set(HEADERS
	hello.h
	)

add_library(hello ${SRCS} ${HEADERS})
