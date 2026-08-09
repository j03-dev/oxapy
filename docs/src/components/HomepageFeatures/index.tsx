import clsx from 'clsx';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

type FeatureItem = {
  title: string;
  description: JSX.Element;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'Fast, built in Rust',
    description: (
      <>
        The core server runs on <code>hyper</code> and <code>tokio</code> with
        URL routing from <code>matchit</code> and JSON handling via{' '}
        <code>orjson</code>. Your Python handlers stay simple while the hot path
        stays in compiled Rust.
      </>
    ),
  },
  {
    title: 'Expressive routing',
    description: (
      <>
        Declare routes with <code>@get</code>, <code>@post</code> and friends,
        use path parameters like <code>{'{name}'}</code>, typed parameters and
        catch-all routes, and group them under routers with a base path.
      </>
    ),
  },
  {
    title: 'Batteries included',
    description: (
      <>
        Middleware, sessions, JWT authentication, CORS, templates, static file
        serving, file uploads and streaming, serializers, application state, and
        hot reload are all built in.
      </>
    ),
  },
];

function Feature({title, description}: FeatureItem) {
  return (
    <div className={clsx('col col--4')}>
      <div className="text--center padding-horiz--md">
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): JSX.Element {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
