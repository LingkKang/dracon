# Base image: Ubuntu latest LTS version
FROM ubuntu:latest 

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update
RUN apt-get install -y curl build-essential gcc make pkg-config libssl-dev

RUN useradd -ms /bin/bash dracon_user

RUN touch /home/dracon_user/.bashrc
RUN echo 'alias cls="clear"' >> /home/dracon_user/.bashrc
RUN echo 'alias la="ls -al"' >> /home/dracon_user/.bashrc
RUN echo \
    'export PS1="\[\033[1;33m\]\u\[\033[0m\]@\[\e[34m\]\[\e[4m\]\W\[\e[m\]\$ "'\
    >> /home/dracon_user/.bashrc

RUN mkdir /proj
RUN chown -R dracon_user:dracon_user /proj
RUN chown -R dracon_user:dracon_user /home/dracon_user
RUN chmod a+x /home/dracon_user/

WORKDIR /proj
USER dracon_user
CMD [ "bash" ]

# Install Rust for the user
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
# As Cargo use its install path for caching, 
# the rust toolchain should be installed for the user, not globally.
