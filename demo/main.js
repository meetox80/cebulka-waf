// demo/main.js
import Fastify from 'fastify'
import Dotenv from 'dotenv'

Dotenv.config()

const _Port = parseInt(process.env.PORT, 10) || 2025;
const _Fastify = Fastify({ logger: true });

_Fastify.get('/', async function Handler(Request, Reply) {
    Reply.header('Content-Type', 'text/plain')
    return "hello world // 0.0.0.0 // asd192.168.0.1 // 92348294192.168.0.123949 // 17f4:779b:713d:9e4b:c55a:893e:ff59:b55b // asdasdad2034917f4:779b:713d:9e4b:c55a:893e:ff59:b55b02349234ads";
})

try {
    await _Fastify.listen({ port: _Port })
} catch (Err) {
    _Fastify.log.error(Err)
    process.exit(1)
}
