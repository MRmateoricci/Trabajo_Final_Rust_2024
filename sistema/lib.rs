#![cfg_attr(not(feature = "std"), no_std, no_main)] 
pub use self::sistema::SistemaRef;

//entregable v2
#[ink::contract]
mod sistema {
    use ink::prelude::vec::Vec;
    use ink::prelude::string::String;
    use ink::prelude::collections::BTreeMap;


    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(
        feature = "std",
        derive(ink::storage::traits::StorageLayout)
    )]
    #[derive(Debug,Clone)]
    pub enum Rol{
        Votante,
        Candidato,
    }

    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(
        feature = "std",
        derive(ink::storage::traits::StorageLayout)
    )]
    #[derive(Debug,Clone)]
    pub struct Usuario{
        nombre:String,
        apellido:String,
        edad:i32,
        dni:i128,
        verificado:bool,
        rol:Option<Rol>,
        acc_id:AccountId
    }
   
    impl Usuario{

        pub fn new(nombre:String,apellido:String,dni:i128,edad:i32,verificado:bool,rol:Option<Rol>,acc_id:AccountId)->Self{
            Self{nombre,apellido,dni,edad,verificado,rol,acc_id}
        }
    }

    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(
        feature = "std",
        derive(ink::storage::traits::StorageLayout)
    )]
    #[derive(Debug,Clone)]
    pub struct Fecha{
        pub dia:u32,
        pub mes:u32,
        pub anio:i32
    }
    impl Fecha {

        #[allow(unused)]
        fn new(dia:u32 , mes:u32, anio:i32) -> Fecha{
            Fecha{dia,mes,anio}
        }

        fn es_fecha_valida(&self)->bool{
            if (self.mes>0)&&(self.mes<=12)&&(self.dia>0)&&(self.dia<=31)&&(self.anio>=1970){
                match self.mes {
                    2=> {
                        if self.is_leap_year(self.anio){
                            return self.dia<=29
                        }
                        else{
                            return self.dia<=28
                        }
                    }
                    1..=12=> {
                        return self.dia<= self.days_in_month(self.mes)
                    }
                    _=> {}
                }
            }
            false
        }

        pub fn to_timestamp(&self) -> Timestamp {
            let days_since_epoch = self.days_since_epoch() as i64;
            let millis_per_day: i64 = 24 * 60 * 60 * 1000;
            days_since_epoch.wrapping_mul(millis_per_day)  as Timestamp
        }
    
        fn days_since_epoch(&self) -> u32 {
            let mut days: u32 = 0;
    
            // Calcular los días desde el Epoch hasta el año actual
            for year in 1970..self.anio {
                days = days.wrapping_add(if self.is_leap_year(year) { 366 } else { 365 });
            }
    
            // Sumar los días de los meses previos en el año actual
            for month in 1..self.mes {
                days = days.wrapping_add(self.days_in_month(month));
            }
    
            // Sumar los días del mes actual
            days = days.wrapping_add(self.dia);
    
            days
        }
    
        fn is_leap_year(&self, year: i32) -> bool {
            (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
        }
    
        fn days_in_month(&self, month: u32) -> u32 {
            match month {
                1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,

                4 | 6 | 9 | 11 => 30,

                2 => {
                    if self.is_leap_year(self.anio) {
                        29
                    } else {
                        28
                    }
                },

                _ => panic!("MES INVALIDO"),
            }
        }

        fn es_mayor(&self, otra_fecha: &Fecha) -> bool {
            if self.anio > otra_fecha.anio {
                true
            } else if self.anio == otra_fecha.anio {
                if self.mes > otra_fecha.mes {
                    true
                } else if self.mes == otra_fecha.mes {
                    if self.dia > otra_fecha.dia {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        
    }
    }
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(
        feature = "std",
        derive(ink::storage::traits::StorageLayout)
    )]
    #[derive(Debug,Clone)]
    pub struct Votacion{
        id:i32,
        puesto:String,
        candidatos:Vec<AccountId>,
        votantes: Vec<AccountId>,
        votos: BTreeMap<AccountId,u32>,    // hashmap con accountid de candidato
        votaron: Vec<AccountId>,
        fecha_inicio:Timestamp,
        fecha_fin:Timestamp,
    }
    impl Votacion{
        pub fn new(id:i32,puesto:String, fecha_inicio:Timestamp, fecha_fin:Timestamp)-> Votacion{
            Votacion {
                id, puesto, candidatos:Vec::new(),votantes:Vec::new(), votos:BTreeMap::new(), votaron:Vec::new(),fecha_inicio, fecha_fin
            }
        }
        
        pub fn inicio(&self, momento:Timestamp)->bool{  
            momento >= self.fecha_inicio
        }

        pub fn finalizo(&self, momento:Timestamp)->bool{ 
            momento >= self.fecha_fin
        }

        pub fn es_votante(&self, acc_id:AccountId)->bool{
            return self.votantes.iter().any(|u| *u == acc_id)
        }

        pub fn es_candidato(&self, acc_id:AccountId)->bool{
            return self.candidatos.iter().any(|u| *u == acc_id)
        }

        pub fn sumar_candidato(&mut self,accid:AccountId){
            self.candidatos.push(accid);
            self.votos.insert(accid, 0);
        }

        pub fn sumar_votante(&mut self,accid:AccountId){
            self.votantes.push(accid);
        }

        pub fn sumar_votador(&mut self,accid:AccountId){
            self.votaron.push(accid);
        }

        pub fn sumar_voto(&mut self,pos:usize){
            self.votos.entry(self.candidatos[pos]).and_modify(|c|* c = c.wrapping_add(1));
        }

        pub fn get_reporte(&self , users:Vec<Usuario>){ //recibe un vec de usuarios porque como no tenemos los datos de cada usuario sino que tenemos su acc id , de algun lado necesitamos agarrar sus datos personales
             
            let total = self.reporte1();
            if total != 0 && self.candidatos.len() > 0  { 
                ink::env::debug_println!("EL TOTAL DE USUARIOS REGISTRADOS Y APROBADOS PARA ESTA VOTACION SON: {}",total);
                let porcentaje = self.reporte2(total);
                if porcentaje != 0.0{
                    ink::env::debug_println!("EL PORCENTAJE DE USUARIOS QUE VOTARON ES:%{}",porcentaje);
                    let ordenado = self.reporte3();
                    for (i,o) in ordenado.iter().enumerate(){
                        let usuario = users.iter().find(| u | u.acc_id == o.0).unwrap(); // podemos hacer unwrap porque sabemos que el usuario existe 
                        ink::env::debug_println!("{}º - {} {}",i.wrapping_add(1) , usuario.nombre , usuario.apellido );
                    }
                }else{
                    panic!("NO HUBIERON VOTOS");
                }
            }else{
                panic!("NO HUBO USUARIOS/CANDIDATOS ACEPTADOS PARA VOTAR");
            }

        }

        fn reporte1(&self) ->usize{
            self.votantes.len()
        }

        fn reporte2(&self, total:usize) ->f64{
            let mut cantvotos: u32=0;
            for v in &self.votos{
                cantvotos = cantvotos.wrapping_add(*v.1);
            };
            if cantvotos !=0 {
                (cantvotos as f64 / total as f64 ) * 100.0
            }
            else{
                 0.0
            }
        }

        fn reporte3(&self) -> Vec<(AccountId,u32)>{
            let aux = self.votos.clone();
            let mut vec: Vec<(AccountId, u32)> = aux.into_iter().collect(); // creamos un vector para ordenar de mayor a menor cada candidato 
            vec.sort_by(|a, b| b.1.cmp(&a.1));
            vec
        }

    
    }

    #[ink(storage)]
    pub struct Sistema {
        nombre_administrador:String,
        espera_usuarios: Vec<Usuario>,
        usuarios_reg: Vec<Usuario>, // hashmap con account id
        espera_candidatos:Vec<(AccountId,i32)>,
        espera_votantes:Vec<(AccountId,i32)>,
        votaciones:Vec<Votacion>,  // hashmap con id de votacion
        admin:AccountId,
    }
    
        impl Sistema {
        //Constructor que recibe unicamente el nombre del administrador
        #[ink(constructor)]
        pub fn new(nombre_administrador: String) -> Self {
            Self { nombre_administrador,espera_usuarios:Vec::new(),espera_candidatos:Vec::new(),espera_votantes:Vec::new(), usuarios_reg:Vec::new(),votaciones: Vec::new(), admin: Self::env().caller() }
        }


        //Crea un usuario verificando que no sea el administrador y que no este repetido y lo agrega a la lista de espera de aprobacion del administrador
        #[ink(message)]
        pub fn registrar_usuario(&mut self, nom:String,apellido:String,edad:i32, dni:i128) {
            let caller= self.env().caller();
            self.registrar_usuario_impl(nom, apellido, edad, dni, caller);
        }

        fn registrar_usuario_impl(&mut self, nom:String,apellido:String,edad:i32, dni:i128, caller:AccountId) {
            if caller != self.admin {  //el administrador no se puede registrar como un usuario 
                let aux: Usuario = Usuario::new(nom, apellido, dni, edad, false, None, caller);
                if edad >= 18 {
                    if  !self.usuarios_reg.iter().any(|u| u.dni == dni || u.acc_id == caller ) {  // no puede haber dos usuarios con el mismo dni 
                        if !self.espera_usuarios.iter().any(|usu| usu.dni == dni|| usu.acc_id == caller){
                            self.espera_usuarios.push(aux);
                        }else{
                            panic!("ESTE USUARIO YA ESTA EN ESPERA DE VALIDACION");
                        }
                        
                    }else{
                        panic!("CUENTA O DNI YA REGISTRADO");
                    }
                }
                else {panic!("NO TIENE LA EDAD SUFICIENTE PARA REGISTRARTE")}
            }
        }


        //Unicamente el administrador puede validar o rechazar un usuario que solicito registrarse
        #[ink(message)] 
        pub fn validar_usuario(&mut self, aceptar: bool) {
            let caller = self.env().caller();
            self.validar_usuario_impl(aceptar,caller);
        }
    
        fn validar_usuario_impl(&mut self, aceptar:bool, caller:AccountId){
            let mut aux: Option<String> = None;
            if caller == self.admin {  // solo el administrador puede validar candidatos 
                if !self.espera_usuarios.is_empty() {  // checkea si hay candidatos a validar, y si hay se empieza a trabajar el primero
                    let mut us = self.espera_usuarios[0].clone();
                    let mut s1 = us.nombre.clone();
                    let s2 = String::from(" ");
                    let s3 = us.apellido.clone();
                    s1.push_str(&s2);
                    s1.push_str(&s3);
                    aux = Some(s1);   
                    if aceptar{  // el admin decide si aceptar o rechazar el candidato
                        us.verificado=true;
                        self.usuarios_reg.push(us)
                    }
                    self.espera_usuarios.remove(0);  // se elimina de la cola de espera de aprobacion 
                }
            }else{
                panic!("SOLO EL ADMINISTRADOR PUEDE VALIDAR USUARIOS");
            }
            if let Some(a) =aux{
                ink::env::debug_println!("Aceptar solicitud de registro del usuario  {:?}",a);
            } else{
                ink::env::debug_println!("No hay solicitudes de registro");
            }
        }


        //Unicamente el administrador puede crear una votacion. No puede haber dos votaciones con el mismo id y las fechas de inicio y fin deben ser validas        #[ink(message)]
        #[ink(message)]
        pub fn crear_votacion(&mut self, id:i32, puesto:String, inicio:Fecha, fin:Fecha) {
            let caller = self.env().caller();
            self.crear_votacion_impl(id, puesto, inicio, fin,caller);
        }

        fn crear_votacion_impl(&mut self, id:i32, puesto:String,fecha_inicio:Fecha,fecha_fin:Fecha, caller:AccountId){ 
            if !fecha_inicio.es_fecha_valida() | !fecha_fin.es_fecha_valida() | !fecha_fin.es_mayor(&fecha_inicio) {
                panic!("FECHA INVALIDA");
            }
            if caller == self.admin {  //solo el administrador puede crear votaciones
                if !self.votaciones.iter().any(|v|v.id==id){  //no se tiene que poder crear dos votaciones con el mismo id
                    let v = Votacion::new(id, puesto, fecha_inicio.to_timestamp(),fecha_fin.to_timestamp());
                    self.votaciones.push(v);       
                    ink::env::debug_println!("fecha inicio: {:?} timestamp: {}",fecha_inicio,fecha_inicio.to_timestamp().wrapping_sub(86_400_000)); //asi comienza ese dia a las 00:00
                    ink::env::debug_println!("fecha fin: {:?} timestamp: {}",fecha_fin,fecha_fin.to_timestamp().wrapping_sub(1));  //asi termina ese dia a las 23:59:59.999
                }

            }
                
        }

        //Los usuarios que se registraron y ya fueron validados por el administrador pueden postularse como candidato o como votante a una votacion (antes de que esta haya comenzado), y esperar a que el administrador los acepte o rechace
        #[ink(message)]
        pub fn postularse_a_votacion(&mut self,rol:Rol, id_de_votacion:i32) {
            let caller = self.env().caller();
            let momento = self.env().block_timestamp();
            self.postularse_a_votacion_impl(rol,id_de_votacion,caller,momento);
        }

        fn postularse_a_votacion_impl(&mut self,rol:Rol, id_de_votacion:i32, caller:AccountId, momento:Timestamp){
                if self.usuarios_reg.iter().any(|u| u.acc_id == caller){   // como el administrador no puede registrarse, si se intenta postular aca va a dar falso
                    if let Some(v) = self.votaciones.iter_mut().find(|vot| vot.id == id_de_votacion){  //si existe la votacion a la que se quiere postular 
                        if v.inicio(momento){
                            panic!("LA VOTACION YA INICIO");
                        }
                        if !v.es_votante(caller) && !v.es_candidato(caller){ // si ya no esta postulado como votante o candidato
                            if !self.espera_candidatos.contains(&(caller,id_de_votacion)) && !self.espera_votantes.contains(&(caller,id_de_votacion)){
                                match rol{ 
                                Rol::Candidato=>{ self.espera_candidatos.push((caller,id_de_votacion)); }, 
                                Rol::Votante=> {  self.espera_votantes.push((caller,id_de_votacion)); }
                                }
                            }
                            else{
                                panic!("ESTAS EN LA COLA DE ESPERA");
                            }
                            
                        }else{
                            panic!("YA TE POSTULASTE A ESTA VOTACION");
                        }
                    }else{
                        panic!("NO EXISTE VOTACION DE ID: {}",id_de_votacion);
                    } 
                } else {
                    panic!("NO ESTAS REGISTRADO O VALIDADO EN EL SISTEMA");
                }
        }

        //Unicamente el administrador puede validar o rechazar candidatos para las votaciones, siempre y cuando esta votacion no haya comenzado
        #[ink(message)]
        pub fn validar_candidato(&mut self, aceptar: bool) {
            let caller = self.env().caller();
            let momento= self.env().block_timestamp();
            self.validar_candidato_impl(aceptar,caller,momento);
        }
        
        fn validar_candidato_impl(&mut self, aceptar:bool, caller:AccountId, momento: Timestamp){
            let mut aux: Option<String> = None;
            let mut vot_id=0;

            if caller == self.admin {  // solo el administrador puede validar candidatos 
                if !self.espera_candidatos.is_empty() {  // checkea si hay candidatos a validar, y si hay se empieza a trabajar el primero
                    let acc_id = self.espera_candidatos[0].0;
                    vot_id = self.espera_candidatos[0].1;
                    
                    self.usuarios_reg.iter_mut().for_each(|u| { //si ya estas en la espera de candidatos, si o si vas a estar en el vector de usuarios 
                        if u.acc_id == acc_id {
                            let mut s1 = u.nombre.clone();
                            let s2 = String::from(" ");
                            let s3 = u.apellido.clone();
                            s1.push_str(&s2);
                            s1.push_str(&s3);
                            aux = Some(s1); 
                            if let Some(vot) = self.votaciones.iter_mut().find(|v| v.id == vot_id){  // va a encontrar la votacion si o si ya que esto se checkea al postularse
                                if vot.inicio(momento){ // Si la votacion ya inicio el administrador no deberia poder aceptarlo o rechazarlo, asique se "descarta" la solicituda de candidato
                                    self.espera_candidatos.remove(0);
                                } else {
                                    if aceptar{  // el admin decide si aceptar o rechazar el candidato
                                        vot.sumar_candidato(acc_id);
                                    }
                                    self.espera_candidatos.remove(0);   // se elimina de la cola de espera de aprobacion 
                                }
                            }
                             
                        }
                    })
                }
            }else{
                panic!("SOLO EL ADMINISTRADOR PUEDE VALIDAR CANDIDATOS");
            }
            if let Some(a) =aux{
                ink::env::debug_println!("Aceptar solicitud de candidato del usuario {:?} para la votacion de id {}",a,vot_id);

            } else{
                ink::env::debug_println!("No hay solicitudes para candidatos");

            }
        }
        //Unicamente el administrador puede validar o rechazar votantes para las votaciones, siempre y cuando esta votacion no haya comenzado
        #[ink(message)]
        pub fn validar_votante(&mut self, aceptar: bool) {
            let caller = self.env().caller();
            let momento = self.env().block_timestamp();
            self.validar_votante_impl(aceptar, caller,momento);
        }

        fn validar_votante_impl(&mut self, aceptar:bool, caller:AccountId, momento: Timestamp){
            let mut aux: Option<String> = None;
            let mut vot_id=0;
            if caller == self.admin {
                if !self.espera_votantes.is_empty() {
                    let acc_id = self.espera_votantes[0].0;
                    vot_id = self.espera_votantes[0].1;
                    
                    self.usuarios_reg.iter_mut().for_each(|u| { //si ya estas en la espera de candidatos, si o si vas a estar en el vector de usuarios 
                        if u.acc_id == acc_id {
                            let mut s1 = u.nombre.clone();
                            let s2 = String::from(" ");
                            let s3 = u.apellido.clone();
                            s1.push_str(&s2);
                            s1.push_str(&s3);
                            aux = Some(s1); 
                            if let Some(vot) = self.votaciones.iter_mut().find(|v| v.id == vot_id){
                                if vot.inicio(momento){ // Si la votacion ya inicio el administrador no deberia poder aceptarlo o rechazarlo, asique se "descarta" la solicituda de votante
                                    self.espera_votantes.remove(0);
                                } else {
                                    if aceptar{  // el admin decide si aceptar o rechazar el votante
                                        vot.sumar_votante(acc_id);
                                    }
                                    self.espera_votantes.remove(0);   // se elimina de la cola de espera de aprobacion 
                                }
                            }
                            
                        }
                    })
                }
            }else{
                panic!("SOLO EL ADMINISTRADOR PUEDE VALIDAR VOTANTES");
            }
            if let Some(a) =aux{
                ink::env::debug_println!("Aceptar solicitud de votante del usuario {:?} para la votacion de id {}",a,vot_id);

            } else{
                ink::env::debug_println!("No hay solicitudes para votante");

            }
        }


        //El votante puede votar validando su identidad (debe estar registrado y validado por el administrador) 
        #[ink(message)]
        pub fn votar(&mut self, id_de_votacion: i32, opcion:i32) {
            let caller = self.env().caller();
            let momento = self.env().block_timestamp();
            self.votar_impl(id_de_votacion, opcion,caller,momento);
        }
        
        fn votar_impl(&mut self,id_de_votacion:i32,opcion:i32, caller:AccountId, momento:Timestamp){
            let mut x: i32  = 0;
            if caller != self.admin{
                if self.usuarios_reg.iter().any(|u| u.acc_id == caller){
                    if let Some(v) = self.votaciones.iter_mut().find(|vot| vot.id == id_de_votacion){
                        if !v.inicio(momento){
                            panic!("LA VOTACION TODAVIA NO INICIO");
                        }
                        if v.finalizo(momento){
                            panic!("LA VOTACION FINALIZO");
                        }
                        if v.es_votante(caller){ //Los candidatos de una votacion no van a poder votar en esa misma ya que no van a estar registrados como votantes 
                            if !v.votaron.contains(&caller){
                                if !v.candidatos.is_empty(){
                                    ink::env::debug_println!("Candidatos");
                                    v.candidatos.iter().for_each(|c|{
                                        x = x.wrapping_add(1);
                                        if let Some(us) =self.usuarios_reg.iter().find(|u|u.acc_id==*c){  //siempre va a entrar ya que si esta como candidato en la votacion si o si esta registrado 
                                            ink::env::debug_println!("Opcion {}: {} {}",x,us.nombre,us.apellido);
                                        }
                                        
                                    });
                                    if let Some(op) = opcion.checked_sub(1) {
                                        if (op as usize) < (v.candidatos.len()) {
                                            v.sumar_voto(op as usize);
                                            v.sumar_votador(caller);
                                        } else {
                                            panic!("OPCION INVALIDA");
                                        }
                                    }
                                }else{
                                    panic!("NO HAY CANDIDATOS POSTULADOS");
                                }
                            }else{
                                panic!("USUARIO YA REALIZO SU VOTO");
                            }
                        }else{
                            panic!("USUARIO NO POSTULADO/APROBADO COMO VOTANTE EN ESTA VOTACION");
                        }
                    }else{
                        panic!("NO EXISTE VOTACION DE ID: {}",id_de_votacion);
                    }
                }else{
                    panic!("USUARIO NO REGISTRADO/APROBADO");
                }
            }else{
                panic!("EL ADMINISTRADOR NO PUEDE VOTAR");
            }

        }

        #[ink(message)]
        pub fn get_votacion(&self, id_de_votacion: i32)->Option<Votacion>{
            if let Some(vot) = self.votaciones.iter().find(|v|v.id==id_de_votacion){
                return Some(vot.clone());
            }
            None
        }
        #[ink(message)]
        pub fn get_usuarios(&self) -> Vec<Usuario>{
            self.usuarios_reg.clone()
        }

    }
        #[cfg(test)]
        mod test{
        use super::*;
        use ink::env:: test;
        use ink::env::test::set_block_timestamp;

        fn default_accounts() -> test::DefaultAccounts<ink::env::DefaultEnvironment> {
            test::default_accounts::<ink::env::DefaultEnvironment>()
        }

        #[ink::test]
        fn test_registrar_usuario() {
            let accounts = default_accounts();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);

            let mut contract = Sistema::new(String::from("admin"));
            let nombre = String::from("John");
            let apellido = String::from("Doe");
            let edad = 25;
            let dni = 12345678;

            contract.registrar_usuario_impl(nombre, apellido, edad, dni,accounts.alice);

            assert_eq!(contract.usuarios_reg.len(), 0);
            assert_eq!(contract.espera_usuarios.len(), 1);
            assert_eq!(contract.espera_usuarios[0].dni, 12345678);
        }
        #[ink::test]
        #[should_panic(expected="ESTE USUARIO YA ESTA EN ESPERA DE VALIDACION" )]
        fn test_intento_registrar_usuario_mismo_dni() {
            let accounts = default_accounts();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);

            let mut contract = Sistema::new(String::from("admin"));
            let nombre = String::from("John");
            let apellido = String::from("Doe");
            let edad = 25;
            let dni = 12345678;

            contract.registrar_usuario_impl(nombre, apellido, edad, dni,accounts.alice);
            contract.registrar_usuario_impl("agus".to_string(), "zap".to_string(), 20, 12345678, accounts.django );

            assert_eq!(contract.espera_usuarios.len(), 1);
        }
        #[ink::test]
        #[should_panic(expected="ESTE USUARIO YA ESTA EN ESPERA DE VALIDACION" )]        
        fn test_intento_registrar_usuario_mismo_account_id() {
            let accounts = default_accounts();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);

            let mut contract = Sistema::new(String::from("admin"));
            let nombre = String::from("John");
            let apellido = String::from("Doe");
            let edad = 25;
            let dni = 12345678;

            contract.registrar_usuario_impl(nombre, apellido, edad, dni,accounts.alice);
            contract.registrar_usuario_impl("agus".to_string(), "zap".to_string(), 20, 123, accounts.alice );

            assert_eq!(contract.espera_usuarios.len(), 1);
        }
        #[ink::test]
        #[should_panic(expected = "CUENTA O DNI YA REGISTRADO")]
        fn test_registrar_usuario_dni_duplicado() {
            let accounts = default_accounts();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
    
            let mut contract = Sistema::new(String::from("admin"));
            let nombre = String::from("John");
            let apellido = String::from("Doe");
            let edad = 25;
            let dni = 12345678;
    
            contract.registrar_usuario_impl(nombre.clone(), apellido.clone(), edad, dni, accounts.alice);
            contract.validar_usuario_impl(true, accounts.bob); //acepta a usuario 1, alice

            contract.registrar_usuario_impl(nombre.clone(), apellido.clone(), edad, dni, accounts.eve);
        }
    
        #[ink::test]
        #[should_panic(expected = "CUENTA O DNI YA REGISTRADO")]
        fn test_registrar_usuario_acc_id_duplicado() {
            let accounts = default_accounts();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
    
            let mut contract = Sistema::new(String::from("admin"));
            let nombre = String::from("John");
            let apellido = String::from("Doe");
            let edad = 25;
            let dni = 12345678;
    
            contract.registrar_usuario_impl(nombre.clone(), apellido.clone(), edad, dni, accounts.alice);
            contract.validar_usuario_impl(true, accounts.bob); //acepta a usuario 1, alice

            contract.registrar_usuario_impl(nombre.clone(), apellido.clone(), edad, 87654321, accounts.alice);
        }
    
        #[ink::test]
        fn test_registrar_usuario_admin() {
            let accounts = default_accounts();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);

            let mut contract = Sistema::new(String::from("admin"));
            let nombre = String::from("John");
            let apellido = String::from("Doe");
            let edad = 25;
            let dni = 12345678;
            contract.registrar_usuario_impl(nombre, apellido, edad, dni,accounts.bob); // account id del admin , no se pushea a la espera

            assert_eq!(contract.espera_usuarios.len(),0);
        }
        #[ink::test]
        #[should_panic(expected="NO TIENE LA EDAD SUFICIENTE PARA REGISTRARTE")]
        fn test_registrar_usuario_menor_de_edad() {
            let accounts = default_accounts();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);

            let mut contract = Sistema::new(String::from("admin"));
            let nombre = String::from("John");
            let apellido = String::from("Doe");
            let edad = 17;
            let dni = 12345678;
            contract.registrar_usuario_impl(nombre, apellido, edad, dni,accounts.alice);

            assert_eq!(contract.espera_usuarios.len(),0);
        }
        #[ink::test]
        fn test_validar_usuario() {
            let accounts = default_accounts();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
    
            let mut contract = Sistema::new(String::from("admin"));
            let nombre = String::from("John");
            let apellido = String::from("Doe");
            let edad = 25;
            let dni = 12345678;
    
            contract.registrar_usuario_impl(nombre.clone(), apellido.clone(), edad, dni, accounts.alice);
            contract.validar_usuario_impl(true, accounts.bob); 

            assert_eq!(contract.usuarios_reg.len(), 1);
         }

        #[ink::test]
        fn test_validar_usuario_no_aceptar() {
            let accounts = default_accounts();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
    
            let mut contract = Sistema::new(String::from("admin"));
            let nombre = String::from("John");
            let apellido = String::from("Doe");
            let edad = 25;
            let dni = 12345678;
    
            contract.registrar_usuario_impl(nombre.clone(), apellido.clone(), edad, dni, accounts.alice);
            contract.validar_usuario_impl(false, accounts.bob); 

            assert_eq!(contract.usuarios_reg.len(), 0);
         }

        #[ink::test]
        fn test_validar_usuario_ninguno() {
            let accounts = default_accounts();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            let mut contract = Sistema::new(String::from("admin"));

            contract.validar_usuario_impl(true, accounts.bob); 
            assert_eq!(contract.usuarios_reg.len(), 0);
         }

        #[ink::test]
        fn test_validar_multiples_usuarios() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice); // Alice es el admin

        let mut contract = Sistema::new(String::from("admin"));
        let nombres = vec!["John", "Jane"];
        let apellidos = vec!["Doe", "Smith"];
        let edad = 25;
        let dnis = vec![12345678, 87654321];

        contract.registrar_usuario_impl(nombres[0].to_string(), apellidos[0].to_string(), edad, dnis[0], accounts.bob);
        contract.registrar_usuario_impl(nombres[1].to_string(), apellidos[1].to_string(), edad, dnis[1], accounts.django);

        contract.validar_usuario_impl(true,accounts.alice);
        contract.validar_usuario_impl(true,accounts.alice);

        assert_eq!(contract.usuarios_reg.len(), 2);
        assert_eq!(contract.usuarios_reg[0].dni, dnis[0]);
        assert_eq!(contract.usuarios_reg[1].dni, dnis[1]);
        assert_eq!(contract.espera_usuarios.len(), 0);
    }
    #[ink::test]
    #[should_panic(expected="SOLO EL ADMINISTRADOR PUEDE VALIDAR USUARIOS")]
    fn test_validar_usuario_no_admin() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);

        let mut contract = Sistema::new(String::from("admin"));
        let nombre = String::from("John");
        let apellido = String::from("Doe");
        let edad = 25;
        let dni = 12345678;

        contract.registrar_usuario_impl(nombre.clone(), apellido.clone(), edad, dni, accounts.frank);
        contract.validar_usuario_impl(true, accounts.alice);
    }
    #[ink::test]
    fn test_crear_votacion(){
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);

        let mut contract = Sistema::new(String::from("admin"));
        let inicio:Fecha = Fecha::new(2,7,2024);
        let fin:Fecha = Fecha::new(15,10,2024);

        set_block_timestamp::<ink::env::DefaultEnvironment>(inicio.to_timestamp());        
        contract.crear_votacion_impl(1, "Presidente".to_string(), inicio , fin , accounts.bob);

        assert_eq!(contract.votaciones.len(), 1);
        assert_eq!(contract.votaciones[0].puesto, "Presidente".to_string());
    }
    #[ink::test]
    fn test_crear_votacion_no_admin(){
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);

        let mut contract = Sistema::new(String::from("admin"));
        let inicio:Fecha = Fecha::new(10,10,2024);
        let fin:Fecha = Fecha::new(15,10,2024);
        contract.crear_votacion_impl(1, "Presidente".to_string(), inicio , fin , accounts.alice);

        assert_eq!(contract.votaciones.len(), 0);
    }
    #[ink::test]
    fn test_crear_votacion_id_duplicado() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice); // Alice es el admin

        let mut contract = Sistema::new(String::from("admin"));
        let id = 1;
        let puesto = String::from("Presidente");
        let inicio = Fecha::new(1, 1, 2024);
        let fin = Fecha::new(31, 12, 2024);

        contract.crear_votacion(id, puesto.clone(), inicio.clone(), fin.clone());
        contract.crear_votacion(id, puesto.clone(), inicio, fin); // Intenta crear una votación con el mismo ID
        assert_eq!(contract.votaciones.len(), 1);
    }
    #[ink::test]
    #[should_panic(expected="FECHA INVALIDA")]
    fn test_crear_votacion_fechas_invalidas() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice); // Alice es el admin

        let mut contract = Sistema::new(String::from("admin"));
        let id = 1;
        let puesto = String::from("Presidente");
        let inicio = Fecha::new(32, 1, 2024); // Fecha inválida
        let fin = Fecha::new(31, 12, 2024);

        contract.crear_votacion(id, puesto.clone(), inicio, fin);
    }

    #[ink::test]
    #[should_panic(expected="FECHA INVALIDA")]
    fn test_crear_votacion_fecha_fin_anterior() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice); // Alice es el admin

        let mut contract = Sistema::new(String::from("admin"));
        let id = 1;
        let puesto = String::from("Presidente");
        let inicio = Fecha::new(31, 12, 2024);
        let fin = Fecha::new(1, 1, 2024); // Fecha de fin anterior a la fecha de inicio

        contract.crear_votacion(id, puesto.clone(), inicio, fin);
    }
    #[ink::test]
    fn test_postularse_a_votacion_exitoso() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice); // Alice es el admin

        let mut contract = Sistema::new(String::from("admin"));
        let id = 1;
        let puesto = String::from("Presidente");
        let inicio = Fecha::new(1, 1, 2025);
        let fin = Fecha::new(31, 12, 2025);

        contract.crear_votacion(id, puesto.clone(), inicio, fin);

        // Registrar y validar a Bob como usuario
        contract.registrar_usuario_impl(String::from("Bob"), String::from("stuart"), 30, 12345678, accounts.bob);
        contract.validar_usuario_impl(true,accounts.alice);

        contract.postularse_a_votacion_impl(Rol::Candidato, id, accounts.bob, Fecha::new(1, 12, 2024).to_timestamp());

        assert_eq!(contract.espera_candidatos.len(), 1);
        assert_eq!(contract.espera_candidatos[0], (accounts.bob, id));

        contract.validar_candidato_impl(true, accounts.alice,Fecha::new(1, 12, 2024).to_timestamp());
        assert_eq!(contract.espera_candidatos.len(), 0);
        assert_eq!(contract.votaciones[0].candidatos[0] ,  accounts.bob);

    }
    #[ink::test]
    #[should_panic(expected="LA VOTACION YA INICIO")]
    fn test_postularse_a_votacion_ya_iniciada() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice); // Alice es el admin

        let mut contract = Sistema::new(String::from("admin"));
        let id = 1;
        let puesto = String::from("Presidente");
        let inicio = Fecha::new(1, 1, 2020);
        let fin = Fecha::new(31, 12, 2025);

        contract.crear_votacion(id, puesto.clone(), inicio, fin);

        // Registrar y validar a Bob como usuario
        contract.registrar_usuario_impl(String::from("Bob"), String::from("stuart"), 30, 12345678, accounts.bob);
        contract.validar_usuario_impl(true,accounts.alice);
        assert_eq!(contract.usuarios_reg.len(),1);

        contract.postularse_a_votacion_impl(Rol::Candidato, id, accounts.bob,Fecha::new(1, 1, 2020).to_timestamp());
    } 

    #[ink::test]
    #[should_panic(expected = "NO ESTAS REGISTRADO O VALIDADO EN EL SISTEMA")]
    fn test_postularse_no_registrado() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob); // Bob no está registrado

        let mut contract = Sistema::new(String::from("admin"));
        let id = 1;
        let puesto = String::from("Presidente");
        let inicio = Fecha::new(1, 1, 2025);
        let fin = Fecha::new(31, 12, 2025);
 
        contract.crear_votacion(id, puesto.clone(), inicio, fin);
        contract.postularse_a_votacion(Rol::Candidato, id);
    }
    #[ink::test]
    #[should_panic(expected = "NO EXISTE VOTACION DE ID: 1")]
    fn test_postularse_votacion_no_existente() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice); 

        let mut contract = Sistema::new(String::from("admin"));

        contract.registrar_usuario_impl(String::from("Bob"), String::from("stuart"), 30, 12345678, accounts.bob);
        contract.validar_usuario_impl(true,accounts.alice);

        contract.postularse_a_votacion_impl(Rol::Candidato, 1, accounts.bob, Fecha::new(1, 12, 2024).to_timestamp());
    }
    #[ink::test]
    #[should_panic(expected = "YA TE POSTULASTE A ESTA VOTACION")]
    fn test_postularse_ya_postulado() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice); // Alice es el admin

        let mut contract = Sistema::new(String::from("admin"));
        let id = 1;
        let puesto = String::from("Presidente");
        let inicio = Fecha::new(1, 1, 2025);
        let fin = Fecha::new(31, 12, 2025);

        contract.crear_votacion(id, puesto.clone(), inicio, fin);

        // Registrar y validar a Bob como usuario
        contract.registrar_usuario_impl(String::from("Bob"), String::from("Builder"), 30, 12345678, accounts.bob);
        contract.validar_usuario_impl(true,accounts.alice);

        contract.postularse_a_votacion_impl(Rol::Candidato, id, accounts.bob, Fecha::new(1, 12, 2024).to_timestamp());
        contract.validar_candidato_impl(true, accounts.alice,Fecha::new(1, 12, 2024).to_timestamp());
        assert_eq!(contract.usuarios_reg.len(),1);

        contract.postularse_a_votacion_impl(Rol::Candidato, id,accounts.bob, Fecha::new(1, 12, 2024).to_timestamp()); // Intentar postularse de nuevo
    }
   
    #[ink::test]
    #[should_panic(expected = "ESTAS EN LA COLA DE ESPERA")]
    fn test_postularse_en_la_cola_de_espera() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice); // Alice es el admin

        let mut contract = Sistema::new(String::from("admin"));
        let id = 1;
        let puesto = String::from("Presidente");
        let inicio = Fecha::new(1, 1, 2022);
        let fin = Fecha::new(30, 12, 2022);

        contract.crear_votacion(id, puesto.clone(), inicio, fin);

        contract.registrar_usuario_impl(String::from("Bob"), String::from("Builder"), 30, 12345678, accounts.bob);
        contract.validar_usuario_impl(true,accounts.alice);

        contract.postularse_a_votacion_impl(Rol::Candidato, id, accounts.bob, Fecha::new(1, 12, 2021).to_timestamp());
        contract.postularse_a_votacion_impl(Rol::Candidato, id,accounts.bob, Fecha::new(1, 12, 2021).to_timestamp());
    }
 
    #[ink::test]
    fn test_validar_candidato_exitoso() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice); // Alice es el admin

        let mut contract = Sistema::new(String::from("admin"));

        // Registrar y validar usuario
        contract.registrar_usuario_impl(String::from("John"), String::from("Doe"), 25, 12345678, accounts.bob);
        contract.validar_usuario_impl(true, accounts.alice);

        // Crear votación
        let inicio = Fecha::new(1, 1, 2024);
        let fin = Fecha::new(10, 1, 2024);
        contract.crear_votacion_impl(1, String::from("Presidente"), inicio, fin, accounts.alice);

        // Postular usuario como candidato
        contract.postularse_a_votacion_impl(Rol::Candidato, 1, accounts.bob, Fecha::new(1, 12, 2023).to_timestamp());

        // Validar candidato
        contract.validar_candidato_impl(true, accounts.alice,Fecha::new(1, 12, 2023).to_timestamp());

        // Verificar que el candidato fue agregado correctamente
        assert!(contract.votaciones[0].es_candidato(accounts.bob));
    }

    #[ink::test]
    #[should_panic(expected = "SOLO EL ADMINISTRADOR PUEDE VALIDAR CANDIDATOS")]
    fn test_validar_candidato_no_admin() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

        let mut contract = Sistema::new(String::from("admin"));

        // Registrar y validar usuario
        contract.registrar_usuario_impl(String::from("John"), String::from("Doe"), 25, 12345678, accounts.bob);
        contract.validar_usuario_impl(true, accounts.alice);

        // Crear votación
        let inicio = Fecha::new(1, 1, 2024);
        let fin = Fecha::new(10, 1, 2024);
        contract.crear_votacion_impl(1, String::from("Presidente"), inicio, fin, accounts.alice);

        // Postular usuario como candidato
        contract.postularse_a_votacion_impl(Rol::Candidato, 1, accounts.bob, Fecha::new(1, 12, 2023).to_timestamp());

        // Intentar validar candidato con un usuario que no es el administrador
        contract.validar_candidato_impl(true, accounts.bob,Fecha::new(1, 12, 2023).to_timestamp());
    } 


    #[ink::test]
    fn validar_candidato_en_votacion_iniciada() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

        let mut contract = Sistema::new(String::from("admin"));

        // Registrar y validar usuario
        contract.registrar_usuario_impl(String::from("John"), String::from("Doe"), 25, 12345678, accounts.bob);
        contract.validar_usuario_impl(true, accounts.alice);

        contract.crear_votacion_impl(1, "Presidente".to_string(),Fecha::new(5, 7, 2024), Fecha::new(1, 1, 2025),accounts.alice);
        contract.postularse_a_votacion_impl(Rol::Candidato, 1, accounts.bob, Fecha::new(1, 12, 2023).to_timestamp());
        
        // Fecha futura donde la votacion ya comenzo 
        let futuro = Fecha::new(10, 7, 2024).to_timestamp();
        
        // Validar candidato cuando la votacion ya comenzo 
        contract.validar_candidato_impl(true, accounts.alice,futuro);

        // Verificaciones
        assert!(contract.espera_candidatos.is_empty());
        assert!(contract.votaciones[0].candidatos.is_empty());     
    }

    #[ink::test]
    #[should_panic(expected = "YA TE POSTULASTE A ESTA VOTACION")]
    fn test_postularse_ya_postulado_votante() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice); // Alice es el admin

        let mut contract = Sistema::new(String::from("admin"));
        let id = 1;
        let puesto = String::from("Presidente");
        let inicio = Fecha::new(1, 1, 2025);
        let fin = Fecha::new(31, 12, 2025);

        contract.crear_votacion(id, puesto.clone(), inicio, fin);

        // Registrar y validar a Bob como usuario
        contract.registrar_usuario_impl(String::from("Bob"), String::from("Builder"), 30, 12345678, accounts.bob);
        contract.validar_usuario_impl(true,accounts.alice);

        // Postularlo y validarlo como votante
        contract.postularse_a_votacion_impl(Rol::Votante, id, accounts.bob, Fecha::new(1, 12, 2024).to_timestamp());
        contract.validar_votante_impl(true, accounts.alice,Fecha::new(1, 12, 2024).to_timestamp());
        assert_eq!(contract.usuarios_reg.len(),1);

        // Intentar postularlo devuelta
        contract.postularse_a_votacion_impl(Rol::Votante, id,accounts.bob, Fecha::new(1, 12, 2024).to_timestamp()); // Intentar postularse de nuevo
    }
   
    #[ink::test]
    #[should_panic(expected = "ESTAS EN LA COLA DE ESPERA")]
    fn test_postularse_en_la_cola_de_espera_votante() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice); // Alice es el admin

        let mut contract = Sistema::new(String::from("admin"));
        let id = 1;
        let puesto = String::from("Presidente");
        let inicio = Fecha::new(1, 1, 2022);
        let fin = Fecha::new(30, 12, 2022);

        contract.crear_votacion(id, puesto.clone(), inicio, fin);

        contract.registrar_usuario_impl(String::from("Bob"), String::from("Builder"), 30, 12345678, accounts.bob);
        contract.validar_usuario_impl(true,accounts.alice);

        contract.postularse_a_votacion_impl(Rol::Votante, id, accounts.bob, Fecha::new(1, 12, 2021).to_timestamp());
        contract.postularse_a_votacion_impl(Rol::Votante, id,accounts.bob, Fecha::new(1, 12, 2021).to_timestamp());
    }
 
    #[ink::test]
    fn test_validar_votante_exitoso() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice); // Alice es el admin

        let mut contract = Sistema::new(String::from("admin"));

        // Registrar y validar usuario
        contract.registrar_usuario_impl(String::from("John"), String::from("Doe"), 25, 12345678, accounts.bob);
        contract.validar_usuario_impl(true, accounts.alice);

        // Crear votación
        let inicio = Fecha::new(1, 1, 2024);
        let fin = Fecha::new(10, 1, 2024);
        contract.crear_votacion_impl(1, String::from("Presidente"), inicio, fin, accounts.alice);

        // Postular usuario como votante
        contract.postularse_a_votacion_impl(Rol::Votante, 1, accounts.bob, Fecha::new(1, 12, 2023).to_timestamp());

        // Validar votante
        contract.validar_votante_impl(true, accounts.alice,Fecha::new(1, 12, 2023).to_timestamp());

        // Verificar que el candidato fue agregado correctamente
        assert!(contract.votaciones[0].es_votante(accounts.bob));
    }

    #[ink::test]
    #[should_panic(expected = "SOLO EL ADMINISTRADOR PUEDE VALIDAR VOTANTES")]
    fn test_validar_votante_no_admin() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

        let mut contract = Sistema::new(String::from("admin"));

        // Registrar y validar usuario
        contract.registrar_usuario_impl(String::from("John"), String::from("Doe"), 25, 12345678, accounts.bob);
        contract.validar_usuario_impl(true, accounts.alice);

        // Crear votación
        let inicio = Fecha::new(1, 1, 2024);
        let fin = Fecha::new(10, 1, 2024);
        contract.crear_votacion_impl(1, String::from("Presidente"), inicio, fin, accounts.alice);

        // Postular usuario como votante
        contract.postularse_a_votacion_impl(Rol::Votante, 1, accounts.bob, Fecha::new(1, 12, 2023).to_timestamp());

        // Intentar validar candidato con un usuario que no es el administrador
        contract.validar_votante_impl(true, accounts.bob,Fecha::new(1, 12, 2023).to_timestamp());
    } 


    #[ink::test]
    fn validar_votante_en_votacion_iniciada() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

        let mut contract = Sistema::new(String::from("admin"));

        // Registrar y validar usuario
        contract.registrar_usuario_impl(String::from("John"), String::from("Doe"), 25, 12345678, accounts.bob);
        contract.validar_usuario_impl(true, accounts.alice);

        contract.crear_votacion_impl(1, "Presidente".to_string(),Fecha::new(5, 7, 2024), Fecha::new(1, 1, 2025),accounts.alice);
        contract.postularse_a_votacion_impl(Rol::Votante, 1, accounts.bob, Fecha::new(1, 12, 2023).to_timestamp());
        
        // Fecha futura donde la votacion ya comenzo 
        let futuro = Fecha::new(10, 7, 2024).to_timestamp();
        
        // Validar votante cuando la votacion ya comenzo 
        contract.validar_votante_impl(true, accounts.alice,futuro);

        assert!(contract.espera_candidatos.is_empty());
        assert!(contract.votaciones[0].candidatos.is_empty());     
    }

    #[ink::test]
    #[should_panic(expected="EL ADMINISTRADOR NO PUEDE VOTAR")]
    fn votar_admin() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

        let mut contract = Sistema::new(String::from("admin"));
        contract.crear_votacion_impl(1, "Presidente".to_string(),Fecha::new(5, 7, 2024), Fecha::new(1, 1, 2025),accounts.alice);
        contract.votar_impl(1, 0,accounts.alice, Fecha::new(6, 7, 2024).to_timestamp()); // el administrador no puede votar
    }

    #[ink::test]
    #[should_panic(expected="USUARIO NO REGISTRADO/APROBADO")]
    fn votar_usuario_no_registrado() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

        let mut contract = Sistema::new(String::from("admin"));
        contract.crear_votacion_impl(1, "Presidente".to_string(),Fecha::new(5, 7, 2024), Fecha::new(1, 1, 2025),accounts.alice);
        contract.votar_impl(1, 0,accounts.bob, Fecha::new(6, 7, 2024).to_timestamp()); 
    }

    #[ink::test]
    #[should_panic(expected="NO EXISTE VOTACION DE ID: 0")]
    fn votar_votacion_no_existente() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

        let mut contract = Sistema::new(String::from("admin"));
        contract.crear_votacion_impl(1, "Presidente".to_string(),Fecha::new(5, 7, 2024), Fecha::new(1, 1, 2025),accounts.alice);
        contract.registrar_usuario_impl(String::from("John"), String::from("Doe"), 25, 12345678, accounts.bob);
        contract.validar_usuario_impl(true, accounts.alice);
        contract.votar_impl(0, 0,accounts.bob, Fecha::new(6, 7, 2024).to_timestamp()); 
    }

    #[ink::test]
    #[should_panic(expected="LA VOTACION TODAVIA NO INICIO")]
    fn votar_votacion_no_comenzo() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

        let mut contract = Sistema::new(String::from("admin"));
        contract.crear_votacion_impl(1, "Presidente".to_string(),Fecha::new(5, 7, 2024), Fecha::new(1, 1, 2025),accounts.alice);
        contract.registrar_usuario_impl(String::from("John"), String::from("Doe"), 25, 12345678, accounts.bob);
        contract.validar_usuario_impl(true, accounts.alice);
        contract.votar_impl(1, 0,accounts.bob, Fecha::new(4, 7, 2024).to_timestamp()); 
    }

    #[ink::test]
    #[should_panic(expected="LA VOTACION FINALIZO")]
    fn votar_votacion_finalizo() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

        let mut contract = Sistema::new(String::from("admin"));
        contract.crear_votacion_impl(1, "Presidente".to_string(),Fecha::new(5, 7, 2024), Fecha::new(1, 1, 2025),accounts.alice);
        contract.registrar_usuario_impl(String::from("John"), String::from("Doe"), 25, 12345678, accounts.bob);
        contract.validar_usuario_impl(true, accounts.alice);
        contract.votar_impl(1, 0,accounts.bob, Fecha::new(2, 1, 2025).to_timestamp()); 
    }

    #[ink::test]
    #[should_panic(expected="USUARIO NO POSTULADO/APROBADO COMO VOTANTE EN ESTA VOTACION")]
    fn votar_sin_postularse() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

        let mut contract = Sistema::new(String::from("admin"));
        contract.crear_votacion_impl(1, "Presidente".to_string(),Fecha::new(5, 7, 2024), Fecha::new(1, 1, 2025),accounts.alice);
        contract.registrar_usuario_impl(String::from("John"), String::from("Doe"), 25, 12345678, accounts.bob);
        contract.validar_usuario_impl(true, accounts.alice);
        contract.votar_impl(1, 0,accounts.bob, Fecha::new(6, 7, 2024).to_timestamp()); 
    }

    #[ink::test]
    #[should_panic(expected="USUARIO NO POSTULADO/APROBADO COMO VOTANTE EN ESTA VOTACION")]
    fn votar_como_candidato() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

        let mut contract = Sistema::new(String::from("admin"));
        contract.crear_votacion_impl(1, "Presidente".to_string(),Fecha::new(5, 7, 2024), Fecha::new(1, 1, 2025),accounts.alice);
        contract.registrar_usuario_impl(String::from("John"), String::from("Doe"), 25, 12345678, accounts.bob);
        contract.validar_usuario_impl(true, accounts.alice);
        contract.postularse_a_votacion_impl(Rol::Candidato, 1, accounts.bob, Fecha::new(4, 7, 2024).to_timestamp());
        contract.validar_candidato_impl(true, accounts.alice, Fecha::new(4, 7, 2024).to_timestamp());
        contract.votar_impl(1, 0,accounts.bob, Fecha::new(6, 7, 2024).to_timestamp()); 
    }

    #[ink::test]
    #[should_panic(expected="NO HAY CANDIDATOS POSTULADOS")]
    fn votar_sin_candidatos() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

        let mut contract = Sistema::new(String::from("admin"));
        contract.crear_votacion_impl(1, "Presidente".to_string(),Fecha::new(5, 7, 2024), Fecha::new(1, 1, 2025),accounts.alice);
        contract.registrar_usuario_impl(String::from("John"), String::from("Doe"), 25, 12345678, accounts.bob);
        contract.validar_usuario_impl(true, accounts.alice);
        contract.postularse_a_votacion_impl(Rol::Votante, 1, accounts.bob, Fecha::new(4, 7, 2024).to_timestamp());
        contract.validar_votante_impl(true, accounts.alice, Fecha::new(4, 7, 2024).to_timestamp());
        contract.votar_impl(1, 1,accounts.bob, Fecha::new(6, 7, 2024).to_timestamp()); //Intenta votar al candidato nro 1 pero este no existe ya que no se postulo ninguno 
        
    }

    #[ink::test]
    #[should_panic(expected="OPCION INVALIDA")]
    fn votar_candidato_invalido() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

        let mut contract = Sistema::new(String::from("admin"));
        contract.crear_votacion_impl(1, "Presidente".to_string(),Fecha::new(5, 7, 2024), Fecha::new(1, 1, 2025),accounts.alice);
        contract.registrar_usuario_impl(String::from("John"), String::from("Doe"), 25, 12345678, accounts.bob);
        contract.validar_usuario_impl(true, accounts.alice);
        contract.postularse_a_votacion_impl(Rol::Votante, 1, accounts.bob, Fecha::new(4, 7, 2024).to_timestamp());
        contract.validar_votante_impl(true, accounts.alice, Fecha::new(4, 7, 2024).to_timestamp());

        contract.registrar_usuario_impl(String::from("Juan"), String::from("Perez"), 23, 123455, accounts.charlie);
        contract.validar_usuario_impl(true, accounts.alice);
        contract.postularse_a_votacion_impl(Rol::Candidato, 1, accounts.charlie, Fecha::new(4, 7, 2024).to_timestamp());
        contract.validar_candidato_impl(true, accounts.alice, Fecha::new(4, 7, 2024).to_timestamp());

        contract.votar_impl(1, 2,accounts.bob, Fecha::new(6, 7, 2024).to_timestamp()); //Intenta votar al candidato nro 2 pero este no existe ya que solo se postulo uno (opcion 1)
    }

    #[ink::test]
    fn votar_exitoso() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

        let mut contract = Sistema::new(String::from("admin"));
        contract.crear_votacion_impl(1, "Presidente".to_string(),Fecha::new(5, 7, 2024), Fecha::new(1, 1, 2025),accounts.alice);
        contract.registrar_usuario_impl(String::from("John"), String::from("Doe"), 25, 12345678, accounts.bob);
        contract.validar_usuario_impl(true, accounts.alice);
        contract.postularse_a_votacion_impl(Rol::Votante, 1, accounts.bob, Fecha::new(4, 7, 2024).to_timestamp());
        contract.validar_votante_impl(true, accounts.alice, Fecha::new(4, 7, 2024).to_timestamp());

        contract.registrar_usuario_impl(String::from("Juan"), String::from("Perez"), 23, 123455, accounts.charlie);
        contract.validar_usuario_impl(true, accounts.alice);
        contract.postularse_a_votacion_impl(Rol::Candidato, 1, accounts.charlie, Fecha::new(4, 7, 2024).to_timestamp());
        contract.validar_candidato_impl(true, accounts.alice, Fecha::new(4, 7, 2024).to_timestamp());

        contract.votar_impl(1, 1,accounts.bob, Fecha::new(6, 7, 2024).to_timestamp()); 

        assert_eq!(contract.votaciones[0].votos, BTreeMap::from([(accounts.charlie,1)]));
    }

    #[ink::test]
    #[should_panic(expected="USUARIO YA REALIZO SU VOTO")]
    fn votar_dos_veces() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

        let mut contract = Sistema::new(String::from("admin"));
        contract.crear_votacion_impl(1, "Presidente".to_string(),Fecha::new(5, 7, 2024), Fecha::new(1, 1, 2025),accounts.alice);
        contract.registrar_usuario_impl(String::from("John"), String::from("Doe"), 25, 12345678, accounts.bob);
        contract.validar_usuario_impl(true, accounts.alice);
        contract.postularse_a_votacion_impl(Rol::Votante, 1, accounts.bob, Fecha::new(4, 7, 2024).to_timestamp());
        contract.validar_votante_impl(true, accounts.alice, Fecha::new(4, 7, 2024).to_timestamp());

        contract.registrar_usuario_impl(String::from("Juan"), String::from("Perez"), 23, 123455, accounts.charlie);
        contract.validar_usuario_impl(true, accounts.alice);
        contract.postularse_a_votacion_impl(Rol::Candidato, 1, accounts.charlie, Fecha::new(4, 7, 2024).to_timestamp());
        contract.validar_candidato_impl(true, accounts.alice, Fecha::new(4, 7, 2024).to_timestamp());

        contract.votar_impl(1, 1,accounts.bob, Fecha::new(6, 7, 2024).to_timestamp()); 
        contract.votar_impl(1, 1,accounts.bob, Fecha::new(6, 7, 2024).to_timestamp()); 

    }

    #[ink::test]
    #[should_panic(expected = "NO HUBO USUARIOS/CANDIDATOS ACEPTADOS PARA VOTAR")]
    fn test_get_reporte_sin_usuarios() {

        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

        let mut contract = Sistema::new(String::from("admin"));          
        contract.crear_votacion_impl(1, "Presidente".to_string(),Fecha::new(5, 7, 2024), Fecha::new(1, 1, 2025),accounts.alice);
        
        if let Some(v) = contract.get_votacion(1){
            v.get_reporte(vec![]);
        }
    }

   #[ink::test]
    #[should_panic(expected = "NO HUBO USUARIOS/CANDIDATOS ACEPTADOS PARA VOTAR")]
    fn test_get_reporte_sin_candidatos_aceptados() {
       
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

        let mut contract = Sistema::new(String::from("admin"));          
        contract.crear_votacion_impl(1, "Presidente".to_string(),Fecha::new(5, 7, 2024), Fecha::new(1, 1, 2025),accounts.alice);
            
        let user1 = Usuario::new("agus".to_string(), "agus".to_string(), 1111, 20, true, Some(Rol::Votante), accounts.bob);
        contract.usuarios_reg.push(user1.clone());
        contract.get_votacion(1).unwrap().votantes.push(user1.acc_id);

        contract.get_votacion(1).unwrap().get_reporte(vec![user1]);
    }

    
#[ink::test]
    #[should_panic(expected = "NO HUBIERON VOTOS")]
    fn test_get_reporte_sin_votos() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

        let mut contract = Sistema::new(String::from("admin"));          
        contract.crear_votacion_impl(1, "Presidente".to_string(),Fecha::new(5, 7, 2024), Fecha::new(1, 1, 2025),accounts.alice);
            
        let user1 = Usuario::new("agus".to_string(), "agus".to_string(), 1111, 20, true, Some(Rol::Votante), accounts.bob);
        let user2 = Usuario::new("agus".to_string(), "agus".to_string(), 2222, 20, true, Some(Rol::Candidato), accounts.django);

        contract.usuarios_reg.push(user1.clone());
        contract.usuarios_reg.push(user2.clone());

        contract.votaciones[0].votantes.push(user1.acc_id);
        contract.votaciones[0].candidatos.push(user2.acc_id);

        contract.get_votacion(1).unwrap().get_reporte(vec![user1,user2]);
    }
#[ink::test]
    fn test_get_reporte_algunos_votos() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

        let mut contract = Sistema::new(String::from("admin"));          
        contract.crear_votacion_impl(1, "Presidente".to_string(),Fecha::new(5, 7, 2024), Fecha::new(1, 1, 2025),accounts.alice);
            
        let user1 = Usuario::new("agus".to_string(), "agus".to_string(), 1111, 20, true, Some(Rol::Votante), accounts.bob);
        let user2 = Usuario::new("agus".to_string(), "agus".to_string(), 2222, 20, true, Some(Rol::Votante), accounts.django);

        let user3 = Usuario::new("agus".to_string(), "agus".to_string(), 333, 20, true, Some(Rol::Candidato), accounts.charlie);
        let user4 = Usuario::new("agus".to_string(), "agus".to_string(), 444, 20, true, Some(Rol::Candidato), accounts.frank);

        contract.usuarios_reg.push(user1.clone());
        contract.usuarios_reg.push(user2.clone());
        contract.usuarios_reg.push(user3.clone());
        contract.usuarios_reg.push(user4.clone());
        contract.votaciones[0].sumar_votante(user1.acc_id);
        contract.votaciones[0].sumar_votante(user2.acc_id);
        contract.votaciones[0].sumar_candidato(user3.acc_id);
        contract.votaciones[0].sumar_candidato(user4.acc_id);
        // Agregar votos
        contract.votaciones[0].sumar_voto(0);
        contract.votaciones[0].get_reporte(contract.get_usuarios());

        // Verificaciones
        assert_eq!(contract.votaciones[0].reporte1(), 2);
        assert_eq!(contract.votaciones[0].reporte2(2), 50.0);
        assert_eq!(contract.votaciones[0].reporte3(), vec![(user3.acc_id, 1), (user4.acc_id,0)]);
    }
#[ink::test]
    fn test_get_reporte_todos_votaron() {
        let accounts = default_accounts();
        test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

        let mut contract = Sistema::new(String::from("admin"));          
        contract.crear_votacion_impl(1, "Presidente".to_string(),Fecha::new(5, 7, 2024), Fecha::new(1, 1, 2025),accounts.alice);
            
        let user1 = Usuario::new("agus".to_string(), "agus".to_string(), 1111, 20, true, Some(Rol::Votante), accounts.bob);
        let user2 = Usuario::new("agus".to_string(), "agus".to_string(), 2222, 20, true, Some(Rol::Votante), accounts.django);

        let user3 = Usuario::new("agus".to_string(), "agus".to_string(), 333, 20, true, Some(Rol::Candidato), accounts.charlie);
        let user4 = Usuario::new("agus".to_string(), "agus".to_string(), 444, 20, true, Some(Rol::Candidato), accounts.frank);

        contract.usuarios_reg.push(user1.clone());
        contract.usuarios_reg.push(user2.clone());
        contract.usuarios_reg.push(user3.clone());
        contract.usuarios_reg.push(user4.clone());
        contract.votaciones[0].sumar_votante(user1.acc_id);
        contract.votaciones[0].sumar_votante(user2.acc_id);
        contract.votaciones[0].sumar_candidato(user3.acc_id);
        contract.votaciones[0].sumar_candidato(user4.acc_id);
        // Agregar votos
        contract.votaciones[0].sumar_voto(0);
        contract.votaciones[0].sumar_voto(0);

        contract.votaciones[0].get_reporte(contract.get_usuarios());

        // Verificaciones
        assert_eq!(contract.votaciones[0].reporte1(), 2);
        assert_eq!(contract.votaciones[0].reporte2(2), 100.0);
        assert_eq!(contract.votaciones[0].reporte3(), vec![(user3.acc_id, 2), (user4.acc_id,0)]);
    }
        
    #[ink::test]
    fn test_es_fecha_valida() {
        let fecha_valida = Fecha::new(10, 5, 2024);
        assert!(fecha_valida.es_fecha_valida());

        let fecha_invalida = Fecha::new(30, 2, 2021);
        assert!(!fecha_invalida.es_fecha_valida());
    }

    #[ink::test]
    fn test_es_bisiesto() {
        let fecha_bisiesta = Fecha::new(1, 1, 2020);
        assert!(fecha_bisiesta.is_leap_year(fecha_bisiesta.anio));

        let fecha_no_bisiesta = Fecha::new(1, 1, 2021);
        assert!(!fecha_no_bisiesta.is_leap_year(fecha_no_bisiesta.anio));
    }

#[ink::test]
    fn test_es_mayor() {
        let fecha1 = Fecha::new(10, 5, 2024);
        let fecha2 = Fecha::new(10, 5, 2023);
        assert!(fecha1.es_mayor(&fecha2));

        let fecha3 = Fecha::new(10, 5, 2024);
        let fecha4 = Fecha::new(10, 4, 2024);
        assert!(fecha3.es_mayor(&fecha4));

        let fecha5 = Fecha::new(10, 5, 2024);
        let fecha6 = Fecha::new(9, 5, 2024);
        assert!(fecha5.es_mayor(&fecha6));
    }
}

}
