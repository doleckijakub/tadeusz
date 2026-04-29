FROM searxng/searxng

COPY settings.yml /etc/searxng/settings.yml
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
