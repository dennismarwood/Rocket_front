FROM rust:1.62

ENV CARGO_TARGET_DIR=docker_target
#Default location for tty
WORKDIR /app

#Bind dir instead
#COPY . /app

RUN cargo install cargo-watch
#RUN cargo install diesel_cli --no-default-features --features mysql

#If I can figure out how to mount from this dockerfile we could save some steps by starting cargo watch when the container is created.
#DOCKER_BUILDKIT=1 docker build -t front:development -f development.Dockerfile .
#RUN --mount=type=bind,source="/home/dennis/front",target=/app
#CMD ["cargo", "watch", "-x", "'run --bin front'"]



#NOTES
#From the project folder, build an image with:
#docker build -t front:development -f development.Dockerfile .

#And then create a container with:
#docker run -p 8000:8000 --rm -itd --mount type=bind,source="$(pwd)",target=/app --name development_front front:development

#Start cargo watch and connect to the container with:
#docker exec -it development_front cargo watch -x 'run --bin front'

#When you are done you can stop the container with:
#docker stop development_front

#DATABASE_URL="mysql://rocket:your_chosen_password@example.com/rocket_app"