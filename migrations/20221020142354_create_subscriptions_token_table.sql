CREATE TABLE subscription_tokens(
  token TEXT NOT NULL,
  subscriber_id UUID NOT NULL REFERENCES subscriptions (id),
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
  PRIMARY KEY (token)
);